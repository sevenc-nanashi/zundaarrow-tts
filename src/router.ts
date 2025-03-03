import { createRouter, createWebHashHistory } from "vue-router";
import LaunchingView from "./views/LaunchingView.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      redirect: "/launch",
    },
    {
      path: "/launch",
      name: "launch",
      component: LaunchingView,
    },
  ],
});
