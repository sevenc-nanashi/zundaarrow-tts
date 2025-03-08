/// <reference types="vite/client" />
/// <reference types="element-plus/global" />

declare module "*.vue" {
  import { defineComponent } from "vue";
  const component: ReturnType<typeof defineComponent>;
  export default component;
}
