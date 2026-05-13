<template>
  <div class="admin-shell">
    <!-- Sidebar -->
    <aside class="sidebar">
      <div class="sidebar-brand">
        <div class="brand-seal">
          <svg viewBox="0 0 40 40" class="brand-seal-svg">
            <rect x="2" y="2" width="36" height="36" rx="3"
              fill="none" stroke="var(--vermillion)" stroke-width="1.5" />
            <text x="12" y="14" text-anchor="middle" dominant-baseline="central" fill="var(--vermillion)"
              font-size="11" font-family="serif">诗</text>
            <text x="28" y="14" text-anchor="middle" dominant-baseline="central" fill="var(--vermillion)"
              font-size="11" font-family="serif">词</text>
            <text x="12" y="28" text-anchor="middle" dominant-baseline="central" fill="var(--vermillion)"
              font-size="11" font-family="serif">管</text>
            <text x="28" y="28" text-anchor="middle" dominant-baseline="central" fill="var(--vermillion)"
              font-size="11" font-family="serif">理</text>
          </svg>
        </div>
        <span class="brand-text">古诗词管理</span>
      </div>

      <nav class="sidebar-nav">
        <router-link
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          class="nav-item"
          :class="{ active: isActive(item.path) }"
        >
          <span class="nav-dot vermillion-dot" v-if="isActive(item.path)"></span>
          <span class="nav-icon" v-html="item.icon"></span>
          <span class="nav-label">{{ item.label }}</span>
        </router-link>
      </nav>

      <div class="sidebar-footer">
        <button class="btn btn-ghost btn-sm" @click="logout" style="width:100%;justify-content:center;">
          退出登录
        </button>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="main-content">
      <div class="content-inner">
        <router-view />
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()

const navItems = [
  { path: '/dashboard', label: '仪表盘', icon: '&#9753;' },
  { path: '/poets',    label: '诗人管理', icon: '&#9733;' },
  { path: '/poems',    label: '诗词管理', icon: '&#9734;' },
  { path: '/users',    label: '用户管理', icon: '&#9787;' },
  { path: '/import',   label: '批量导入', icon: '&#9744;' },
  { path: '/export',   label: '数据导出', icon: '&#9745;' },
]

function isActive(path: string): boolean {
  return route.path === path || (path !== '/dashboard' && route.path.startsWith(path))
}

function logout() {
  localStorage.removeItem('admin_token')
  router.push('/login')
}
</script>

<style scoped>
.admin-shell {
  display: flex;
  min-height: 100vh;
}

/* ===== Sidebar ===== */
.sidebar {
  width: 230px;
  min-width: 230px;
  background: var(--ink-deep);
  border-right: 1px solid var(--ink-border);
  display: flex;
  flex-direction: column;
  position: fixed;
  top: 0;
  left: 0;
  bottom: 0;
  z-index: 50;
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 22px 20px 18px;
  border-bottom: 1px solid var(--ink-border);
}

.brand-seal-svg {
  width: 36px;
  height: 36px;
}

.brand-text {
  font-family: 'Noto Serif SC', serif; font-weight: 600;
  font-size: var(--text-lg);
  color: var(--paper-bright);
  letter-spacing: 0.08em;
}

.sidebar-nav {
  flex: 1;
  padding: 12px 10px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border-radius: var(--radius-md);
  color: var(--paper-dim);
  font-family: 'Noto Serif SC', serif; font-weight: 600;
  font-size: var(--text-base);
  text-decoration: none;
  transition: all var(--duration-fast);
  position: relative;
}

.nav-item:hover {
  color: var(--paper-bright);
  background: var(--ink-surface);
}

.nav-item.active {
  color: var(--paper-bright);
  background: var(--ink-raised);
}

.nav-item.active::after {
  content: '';
  position: absolute;
  left: 0;
  top: 8px;
  bottom: 8px;
  width: 2px;
  background: var(--vermillion);
  border-radius: 0 1px 1px 0;
}

.nav-icon {
  font-size: 14px;
  width: 20px;
  text-align: center;
  opacity: 0.6;
}

.nav-item.active .nav-icon {
  opacity: 1;
  color: var(--gold);
}

.nav-dot {
  position: absolute;
  left: -6px;
  top: 50%;
  transform: translateY(-50%);
}

.sidebar-footer {
  padding: 12px 16px;
  border-top: 1px solid var(--ink-border);
}

/* ===== Main Content Area ===== */
.main-content {
  flex: 1;
  margin-left: 230px;
  background: var(--ink-base);
  min-height: 100vh;
}

.content-inner {
  padding: 32px 36px;
  max-width: 1280px;
}
</style>
