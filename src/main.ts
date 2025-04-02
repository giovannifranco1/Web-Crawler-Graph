import { createApp } from "vue";
import HomeView from "./views/Home.vue";
import GraphView from "./views/Graph.vue";
import App from "./App.vue";
import "./assets/css.css";

import { createRouter, createWebHashHistory } from 'vue-router'

const routes = [
    { path: '/', component: HomeView },
    { path: '/result', component: GraphView },
  ]
  
  const router = createRouter({
    history: createWebHashHistory(),    
    routes,
  });

createApp(App)
.use(router)
.mount("#app");
