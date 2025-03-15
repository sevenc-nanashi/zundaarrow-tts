fn main() -> anyhow::Result<()> {
    let build = vergen_gitcl::BuildBuilder::all_build()?;
    let gitcl = vergen_gitcl::GitclBuilder::all_git()?;
    let rustc = vergen_gitcl::RustcBuilder::all_rustc()?;

    vergen_gitcl::Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&gitcl)?
        .add_instructions(&rustc)?
        .emit()?;

    tauri_build::build();

    Ok(())
}
