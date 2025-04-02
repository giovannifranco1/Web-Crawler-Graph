<script setup lang="ts">
import { ref, computed } from 'vue';

defineEmits(['open-url']);
const pageUrl = ref("");
const isLoading = ref(false);

const isValidUrl = computed(() => {
    if (!pageUrl.value) return true;
    try {
        new URL(pageUrl.value);
        return true;
    } catch {
        return false;
    }
});

function handleSubmit() {
    if (!pageUrl.value || !isValidUrl.value) return;
    isLoading.value = true;
    setTimeout(() => {
        isLoading.value = false;
    }, 500);
}
</script>

<template>
    <div class="container">
        <div class="header">
            <div class="logo">
                <svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="logo-icon">
                    <circle cx="12" cy="12" r="10"></circle>
                    <line x1="2" y1="12" x2="22" y2="12"></line>
                    <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
                </svg>
                <h1 class="py-5">Graphix</h1>
            </div>
            <p class="subtitle">Visualize and analyze web page structures</p>
            <div class="features">
                <div class="feature">
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3Z"></path>
                    </svg>
                    <span>Visualize DOM</span>
                </div>
                <div class="feature">
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"></polyline>
                    </svg>
                    <span>Analyze Performance</span>
                </div>
                <div class="feature">
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M12 20.94c1.5 0 2.75 1.06 4 1.06 3 0 6-8 6-12.22A4.91 4.91 0 0 0 17 5c-2.22 0-4 1.44-5 2-1-.56-2.78-2-5-2a4.9 4.9 0 0 0-5 4.78C2 14 5 22 8 22c1.25 0 2.5-1.06 4-1.06Z"></path>
                        <path d="M10 2c1 .5 2 2 2 5"></path>
                    </svg>
                    <span>SEO Insights</span>
                </div>
            </div>
        </div>

        <form @submit.prevent="handleSubmit(); $emit('open-url', pageUrl)" class="url-form">
            <div class="form-group">
                <div class="input-wrapper" :class="{ 'error': pageUrl && !isValidUrl }">
                    <input 
                        type="text" 
                        v-model="pageUrl" 
                        id="url-input" 
                        placeholder="https://example.com" 
                        autocomplete="url"
                        :disabled="isLoading"
                    />
                    <span v-if="pageUrl && !isValidUrl" class="error-message">Please enter a valid URL</span>
                </div>
                <button 
                    type="submit" 
                    :disabled="!pageUrl || !isValidUrl || isLoading"
                    :class="{ 'loading': isLoading }"
                >
                    <span v-if="!isLoading">Analyze</span>
                    <span v-else class="loader"></span>
                </button>
            </div>
            
            <div class="info-text">
                Enter a complete URL including http:// or https:// to visualize its structure
            </div>
        </form>
        
        <div class="recent-urls">
            <h3>Recent URLs</h3>
            <div class="url-list">
                <div class="url-item">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20Z"></path>
                        <path d="m4.9 4.9 14.2 14.2"></path>
                    </svg>
                    <span>example.com</span>
                </div>
                <div class="url-item">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20Z"></path>
                        <path d="m4.9 4.9 14.2 14.2"></path>
                    </svg>
                    <span>github.com</span>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.container {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem 1rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2rem;
}

.header {
    text-align: center;
}

.logo {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
}

.logo-icon {
    color: #3b82f6;
}

h1 {
    font-size: 3.5rem;
    font-weight: 700;
    background: linear-gradient(45deg, #3b82f6, #8b5cf6);
    background-clip: text;
    color: transparent;
    margin: 0;
}

.subtitle {
    color: #6b7280;
    font-size: 1.2rem;
    margin-bottom: 1.5rem;
}

.features {
    display: flex;
    justify-content: center;
    gap: 2rem;
    margin-top: 1rem;
}

.feature {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: #4b5563;
}

.feature svg {
    color: #3b82f6;
}

.url-form {
    width: 100%;
    max-width: 600px;
}

.form-group {
    display: flex;
    gap: 0.75rem;
    width: 100%;
    margin-bottom: 0.5rem;
}

.input-wrapper {
    flex: 1;
    position: relative;
}

.form-group input {
    width: 100%;
    padding: 0.75rem 1rem;
    border-radius: 0.5rem;
    font-size: 1rem;
    transition: all 0.2s;
}

.input-wrapper.error input {
    border-color: #ef4444;
}

.error-message {
    position: absolute;
    left: 0;
    bottom: -1.5rem;
    color: #ef4444;
    font-size: 0.875rem;
}

.form-group button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 0.5rem;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    min-width: 100px;
    display: flex;
    justify-content: center;
    align-items: center;
}


.form-group button:disabled {
    cursor: not-allowed;
}

.info-text {
    color: #6b7280;
    font-size: 0.875rem;
    text-align: center;
    margin-top: 1rem;
}

.loader {
    width: 1rem;
    height: 1rem;
    border: 2px solid #ffffff;
    border-bottom-color: transparent;
    border-radius: 50%;
    display: inline-block;
    animation: rotation 1s linear infinite;
}

.recent-urls {
    width: 100%;
    max-width: 600px;
    margin-top: 1rem;
}

.recent-urls h3 {
    font-size: 1.2rem;
    color: #e2e8f0;
    margin-bottom: 0.75rem;
    text-align: center;
}

.url-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

.url-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background-color: #1e293b;
    border-radius: 0.5rem;
    cursor: pointer;
    transition: background-color 0.2s;
}

.url-item:hover {
    background-color: #334155;
}

.url-item svg {
    color: #94a3b8;
}

.url-item span {
    color: #e2e8f0;
}

@keyframes rotation {
    0% {
        transform: rotate(0deg);
    }
    100% {
        transform: rotate(360deg);
    }
}

@media (max-width: 640px) {
    .form-group {
        flex-direction: column;
    }
    
    .form-group button {
        width: 100%;
        margin-top: 1.5rem;
    }
    
    .error-message {
        position: static;
        margin-top: 0.25rem;
    }
    
    .features {
        flex-direction: column;
        gap: 0.75rem;
        align-items: center;
    }
}
</style>