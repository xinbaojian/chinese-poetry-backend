<template>
  <div class="login-scene">
    <!-- Ink wash mountain backdrop -->
    <div class="ink-mountains" aria-hidden="true">
      <div class="mountain m1"></div>
      <div class="mountain m2"></div>
      <div class="mountain m3"></div>
      <div class="mist"></div>
    </div>

    <div class="login-panel">
      <!-- Vermilion seal mark -->
      <div class="seal">
        <svg viewBox="0 0 64 64" class="seal-svg">
          <rect x="3" y="3" width="58" height="58" rx="3"
            fill="none" stroke="var(--vermillion)" stroke-width="2" />
          <text x="17" y="26" text-anchor="middle" dominant-baseline="central" fill="var(--vermillion)"
            font-size="16" font-family="serif">诗</text>
          <text x="47" y="26" text-anchor="middle" dominant-baseline="central" fill="var(--vermillion)"
            font-size="16" font-family="serif">词</text>
          <text x="17" y="50" text-anchor="middle" dominant-baseline="central" fill="var(--vermillion)"
            font-size="16" font-family="serif">管</text>
          <text x="47" y="50" text-anchor="middle" dominant-baseline="central" fill="var(--vermillion)"
            font-size="16" font-family="serif">理</text>
        </svg>
      </div>

      <h1 class="login-title">古诗词管理后台</h1>
      <p class="login-subtitle">青灯黄卷 · 诗书传家</p>

      <form class="login-form" @submit.prevent="handleLogin">
        <div class="form-group">
          <label class="form-label" for="username">帐号</label>
          <input
            id="username"
            v-model="form.username"
            class="input"
            placeholder="请输入帐号"
            autocomplete="username"
          />
        </div>
        <div class="form-group">
          <label class="form-label" for="password">密码</label>
          <input
            id="password"
            v-model="form.password"
            class="input"
            type="password"
            placeholder="请输入密码"
            autocomplete="current-password"
          />
        </div>
        <button
          type="submit"
          class="btn btn-primary login-btn"
          :disabled="loading"
        >
          <span v-if="!loading">进入</span>
          <span v-else class="spinner"></span>
        </button>
      </form>

      <div class="login-footer">
        <span class="ink-brush-line"></span>
        <span class="footer-text">翰墨书香 · 诗韵悠长</span>
        <span class="ink-brush-line"></span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import axios from 'axios'
import { toast } from '../utils/toast'

const router = useRouter()
const loading = ref(false)

const form = reactive({
  username: '',
  password: '',
})

async function handleLogin() {
  if (!form.username || !form.password) {
    toast.warning('请输入帐号和密码')
    return
  }
  loading.value = true
  try {
    const res = await axios.post('/api/v1/admin/login', form)
    localStorage.setItem('admin_token', res.data.token)
    toast.success('登录成功')
    router.push('/dashboard')
  } catch (e: any) {
    const msg = e.response?.data?.error || '登录失败'
    toast.error(msg)
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.login-scene {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
  background: var(--ink-deepest);
}

/* ===== Ink Wash Mountains ===== */
.ink-mountains {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 0;
}

.mountain {
  position: absolute;
  bottom: 0;
  width: 100%;
  opacity: 0.06;
}

.m1 {
  height: 45%;
  background:
    radial-gradient(ellipse 120% 100% at 20% 100%, #fff 0%, transparent 70%),
    radial-gradient(ellipse 80% 100% at 80% 100%, #fff 0%, transparent 70%);
}

.m2 {
  height: 30%;
  background:
    radial-gradient(ellipse 150% 100% at 50% 100%, #fff 0%, transparent 65%);
  opacity: 0.04;
  bottom: 5%;
}

.m3 {
  height: 20%;
  background:
    radial-gradient(ellipse 100% 100% at 30% 100%, #fff 0%, transparent 60%),
    radial-gradient(ellipse 60% 100% at 70% 100%, #fff 0%, transparent 60%);
  opacity: 0.03;
  bottom: 10%;
}

.mist {
  position: absolute;
  bottom: 0;
  width: 100%;
  height: 40%;
  background: linear-gradient(to top, var(--ink-deepest) 0%, transparent 100%);
}

/* ===== Login Panel ===== */
.login-panel {
  position: relative;
  z-index: 1;
  width: 400px;
  padding: 44px 40px 32px;
  background: var(--ink-surface);
  border: 1px solid var(--ink-border);
  border-radius: var(--radius-lg);
  box-shadow:
    0 0 0 1px var(--ink-border),
    0 24px 64px rgba(0, 0, 0, 0.5);
  animation: inkBleed 0.5s var(--ease-out-expo);
}

/* ===== Seal Mark ===== */
.seal {
  display: flex;
  justify-content: center;
  margin-bottom: 16px;
}

.seal-svg {
  width: 52px;
  height: 52px;
  animation: sealStamp 0.6s 0.2s var(--ease-spring) both;
}

.seal-svg text {
  letter-spacing: 2px;
}

/* ===== Titles ===== */
.login-title {
  text-align: center;
  font-size: var(--text-2xl);
  color: var(--paper-bright);
  margin-bottom: 4px;
  letter-spacing: 0.15em;
  animation: slideUp 0.4s 0.3s var(--ease-out-expo) both;
}

.login-subtitle {
  text-align: center;
  font-size: var(--text-sm);
  color: var(--paper-dim);
  margin-bottom: 36px;
  letter-spacing: 0.2em;
  animation: slideUp 0.4s 0.4s var(--ease-out-expo) both;
}

/* ===== Form ===== */
.login-form {
  animation: slideUp 0.4s 0.5s var(--ease-out-expo) both;
}

.login-form .form-group {
  margin-bottom: 20px;
}

.login-btn {
  width: 100%;
  justify-content: center;
  padding: 12px;
  font-size: var(--text-lg);
  letter-spacing: 0.2em;
  margin-top: 8px;
}

/* ===== Spinner ===== */
.spinner {
  width: 18px;
  height: 18px;
  border: 2px solid rgba(255,255,255,0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
  display: inline-block;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* ===== Footer ===== */
.login-footer {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 24px;
  animation: slideUp 0.4s 0.6s var(--ease-out-expo) both;
}

.ink-brush-line {
  flex: 1;
  height: 1px;
  background: linear-gradient(to right, transparent, var(--ink-border), transparent);
}

.footer-text {
  font-family: 'Noto Serif SC', serif; font-weight: 600;
  font-size: var(--text-xs);
  color: var(--paper-faint);
  letter-spacing: 0.2em;
  white-space: nowrap;
}
</style>
