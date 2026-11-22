import { createApp } from 'vue';
import { createPinia } from 'pinia';
import { createRouter, createWebHistory } from 'vue-router';
import App from './App.vue';
import DashboardView from './components/DashboardView.vue';
import ModelsView from './components/ModelsView.vue';
import PreviewView from './components/PreviewView.vue';
import SettingsView from './components/SettingsView.vue';
import './style.css';

const pinia = createPinia();

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: DashboardView },
    { path: '/models', component: ModelsView },
    { path: '/preview', component: PreviewView },
    { path: '/settings', component: SettingsView },
  ],
});

const app = createApp(App);
app.use(pinia);
app.use(router);
app.mount('#app');