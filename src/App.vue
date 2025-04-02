<script setup lang="ts">
import { provide, ref } from 'vue';

const globalUrls = ref([]);

provide("global-urls", globalUrls);

</script>

<template>
    <RouterView/>
</template>

<style scoped>
.logo.vite:hover {
    filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
    filter: drop-shadow(0 0 2em #249b73);
}
</style>

<style scoped>

.title-bar-controls {
    display: flex;
    justify-content: flex-end;
    gap: 0.5em;
    margin-top: 1em;
    margin-right: 1em;
}

.title-bar-controls-item {
    color: #fff;
    cursor: pointer;
    width: 60px;
    min-width: 20px;
    transition: background-color 0.25s;
}

.content-box {
    /* margin-top: -10em;
    width: 100%;
    max-width:  720px; */
}

</style>

<style>

:root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;
    color: #0f0f0f;
    background-color: #f6f6f6;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
}

html, body {
    height: 100%;
    margin: 0;
    padding: 0;
    overflow: hidden; /* Remove qualquer scroll extra */
}

.container {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    text-align: center;
    min-height: 100vh; /* Garante que ocupe toda a altura da tela */
    width: 100%; /* Garante que ocupe toda a largura disponível */
    max-width: 100%; /* Evita overflow horizontal */
    overflow: hidden; /* Evita scroll indesejado */
    position: relative; /* Estabelece um contexto de posicionamento */
    margin: 0 auto; /* Centraliza horizontalmente */
    padding: 1rem; /* Adiciona um padding interno para melhor espaçamento */
    box-sizing: border-box; /* Garante que padding não afete a largura total */
    -webkit-app-region: drag; /* Permite arrastar a janela */
}

.container > * {
    -webkit-app-region: no-drag; /* Permite interagir com elementos dentro do container */
    max-width: 100%; /* Garante que os filhos não ultrapassem a largura do container */
}

.logo {
    height: 6em;
    padding: 1.5em;
    will-change: filter;
    transition: 0.75s;
}

.logo.tauri:hover {
    filter: drop-shadow(0 0 2em #24c8db);
}


a {
    font-weight: 500;
    color: #646cff;
    text-decoration: inherit;
}

a:hover {
    color: #535bf2;
}

h1 {
    text-align: center;
}

input,
button {
    border-radius: 8px;
    border: 1px solid transparent;
    padding: 0.6em 1.2em;
    font-size: 1em;
    font-weight: 500;
    font-family: inherit;
    color: #0f0f0f;
    background-color: #ffffff;
    transition: border-color 0.25s;
    box-shadow: 0 2px 2px rgba(255, 0, 0, 0.9);
}

button {
    cursor: pointer;
}

button:hover {
    border-color: #396cd8;
}
button:active {
    border-color: #396cd8;
    background-color: #e8e8e8;
}

input,
button {
    outline: none;
}

@media (prefers-color-scheme: dark) {
    :root {
        color: #f6f6f6;
        background-color: #0A0A0A;
    }

    a:hover {
        color: #24c8db;
    }

    input,
    button {
        color: #ffffff;
        background-color: #2A3439;
    }
    button:active {
        background-color: #0f0f0f69;
    }
}
</style>
