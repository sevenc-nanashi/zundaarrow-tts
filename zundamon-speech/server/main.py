from __future__ import annotations

import tempfile
import os
from typing import Optional
import soundfile as sf
import fastapi
from fastapi.middleware.cors import CORSMiddleware
import pydantic
import threading
import sys
import io
import nltk
import json
from functools import partialmethod
from tqdm import tqdm


tqdm.__init__ = partialmethod(tqdm.__init__, disable=True)
nltk.download("averaged_perceptron_tagger")
nltk.download("averaged_perceptron_tagger_eng")

zunspeech_dir = os.path.dirname(os.path.abspath(__file__)) + "/../zundamon-speech-webui"
gpt_sovits_dir = os.path.join(zunspeech_dir, "GPT-SoVITS")
sys.path.insert(0, zunspeech_dir)
sys.path.insert(0, gpt_sovits_dir)
sys.path.append(os.path.join(gpt_sovits_dir, "GPT_SoVITS"))

server_dir = os.path.dirname(os.path.abspath(__file__))

os.chdir(gpt_sovits_dir)
gpt_path = (
    f"{server_dir}/../zundamon_GPT-SoVITS/GPT_weights_v2/zudamon_style_1-e15.ckpt"
)
sovits_path = (
    f"{server_dir}/../zundamon_GPT-SoVITS/SoVITS_weights_v2/zudamon_style_1_e8_s96.pth"
)
with open("./weight.json", "w") as f:
    json.dump(
        {
            "GPT": {
                "v2": gpt_path,
            },
            "SoVITS": {
                "v2": sovits_path,
            },
        },
        f,
    )


# Import your inference functions and required packages (adjust import paths as needed)
from GPT_SoVITS.inference_webui import (
    get_tts_wav,
)

REF_AUDIO_PATH = zunspeech_dir + "/reference/reference.wav"
REF_TEXT_PATH = zunspeech_dir + "/reference/ref_text.txt"

REFERENCE_LANGUAGE_MAP = {
    "zh": "Chinese",
    "en": "English",
    "ja": "Japanese",
    "yue": "Cantonese",
    "ko": "Korean",
}
TARGET_LANGUAGE_MAP = {
    "auto": "Multilingual Mixed",
    "zh": "Chinese",
    "en": "English",
    "ja": "Japanese",
    "yue": "Cantonese",
    "ko": "Korean",
    "zh+en": "Chinese-English Mixed",
    "ja+en": "Japanese-English Mixed",
    "yue+en": "Cantonese-English Mixed",
    "ko+en": "Korean-English Mixed",
}

app = fastapi.FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:1420", "http://tauri.localhost"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


class TTSRequest(pydantic.BaseModel):
    text: str
    target_language: str
    reference_text: Optional[str] = None
    reference_language: Optional[str] = None


@app.get("/")
def read_root():
    return {"code": "ok"}


synthesis_lock = threading.Lock()


@app.post("/tts")
async def tts(request: fastapi.Request):
    request_data: TTSRequest
    ref_audio: fastapi.UploadFile | None
    if request.headers.get("Content-Type") == "multipart/form-data":
        boundary = request.headers["Content-Type"].split("=")[1]
        form_data = fastapi.FormData(boundary=boundary)
        form_data = form_data.parse(await request.body())
        request_data = TTSRequest.parse_raw(form_data["data"])
        ref_audio = form_data["ref_audio"]
    elif request.headers.get("Content-Type") == "application/json":
        request_data = TTSRequest.parse_obj(await request.json())
        ref_audio = None
    else:
        return {"error": "Content-Type not supported"}, 400

    if (
        bool(request_data.reference_language)
        != bool(request_data.reference_text)
        != bool(ref_audio)
    ):
        raise fastapi.HTTPException(
            status_code=400,
            detail="reference_language, reference_text, and ref_audio must be all present or all absent",
        )

    if request_data.target_language not in TARGET_LANGUAGE_MAP:
        raise fastapi.HTTPException(status_code=400, detail="Invalid target_language")

    if (
        request_data.reference_language
        and request_data.reference_language not in REFERENCE_LANGUAGE_MAP
    ):
        raise fastapi.HTTPException(
            status_code=400, detail="Invalid reference_language"
        )

    ref_audio_path = REF_AUDIO_PATH
    ref_text: str
    ref_language: str
    if request_data.reference_language:
        ref_text = request_data.reference_text
        ref_language = REFERENCE_LANGUAGE_MAP[request_data.reference_language]
    else:
        ref_language = REFERENCE_LANGUAGE_MAP["ja"]
        with open(REF_TEXT_PATH, "r", encoding="utf-8") as file:
            ref_text = file.read()

    if ref_audio:
        with tempfile.NamedTemporaryFile(delete=False) as temp_audio:
            temp_audio.write(ref_audio.file.read())
            ref_audio_path = temp_audio.name

    with synthesis_lock:
        synthesis_result = get_tts_wav(
            ref_wav_path=ref_audio_path,
            prompt_text=ref_text,
            prompt_language=ref_language,
            text=request_data.text,
            text_language=TARGET_LANGUAGE_MAP[request_data.target_language],
            top_p=1,
            temperature=1,
        )

        result_list = list(synthesis_result)

        if result_list:
            last_sampling_rate, last_audio_data = result_list[-1]
            output = io.BytesIO()
            sf.write(output, last_audio_data, last_sampling_rate, format="wav")
            output.seek(0)
            return fastapi.responses.StreamingResponse(output, media_type="audio/wav")
        else:
            raise fastapi.HTTPException(status_code=500, detail="Synthesis failed")


if __name__ == "__main__":
    import uvicorn

    if len(sys.argv) < 2:
        raise ValueError("Port number not provided")
    port = int(sys.argv[1])

    uvicorn.run(app, host="localhost", port=port)
