<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import FormOpenUrl from './FormOpenUrl.vue';
import { ref } from 'vue';

const isLoading = ref(false);

class Graph {
    url: string;
    sub_urls: Graph[];

    constructor(url: string, sub_urls: Graph[]) {
        this.url = url;
        this.sub_urls = sub_urls;
    }
}


// Methods
async function openUrl(url: string) {
    isLoading.value = true;

    try {
        const result = await invoke("open", { url }) as Graph;
        console.log("Retorno de invoke:", result, JSON.stringify(result));

        if (result) {
            localStorage.setItem('global-urls', JSON.stringify(result));
        }
    } finally {
        isLoading.value = false;
    }
}

</script>

<template>    

    <main class="container">
        <div class="content-box">
            <div class="relative">
                <FormOpenUrl v-if="!isLoading" @open-url="(url) => openUrl(url)" />
                
                <div v-if="isLoading" class="fixed inset-0 bg-black/90 backdrop-blur-sm flex items-center justify-center z-50">
                    <div class="bg-zinc-900 p-8 rounded-2xl shadow-2xl transform transition-all duration-300 border border-red-700/50">
                        <div class="flex flex-col items-center space-y-6">
                            <div class="relative">
                                <!-- Spinner externo -->
                                <div class="w-24 h-24 rounded-full border-8 border-red-800/20 animate-pulse"></div>
                                <div class="w-24 h-24 rounded-full border-8 border-red-600 border-t-transparent animate-spin absolute top-0 left-0"></div>
                                
                                <!-- Spinner interno -->
                                <div class="absolute inset-0 flex items-center justify-center">
                                    <div class="w-12 h-12 rounded-full border-4 border-red-800/20 animate-pulse"></div>
                                    <div class="w-12 h-12 rounded-full border-4 border-red-500 border-t-transparent animate-spin-reverse absolute"></div>
                                </div>
                            </div>
                            <div class="text-center">
                                <h3 class="text-2xl font-bold text-red-500">Gerando Links</h3>
                                <p class="text-sm text-gray-400 mt-2">Isso pode levar alguns segundos...</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </main>

</template>

<style>
@keyframes spin-reverse {
    from {
        transform: rotate(360deg);
    }
    to {
        transform: rotate(0deg);
    }
}

.animate-spin-reverse {
    animation: spin-reverse 3s linear infinite;
}

@keyframes pulse {
    0%, 100% {
        opacity: 1;
    }
    50% {
        opacity: 0.3;
    }
}

.animate-pulse {
    animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
</style>