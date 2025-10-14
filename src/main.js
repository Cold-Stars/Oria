import { createApp } from "vue";
import App from "./App.vue";
import naive from 'naive-ui';
import './style.css';

const app = createApp(App);

// 配置 Naive UI（但不强制主题，避免与浅色主题冲突）
app.use(naive);

app.mount("#app");
