<template>
  <div>
    <!-- Welcome Banner -->
    <div class="welcome-banner">
      <div class="welcome-text">
        <h2 class="welcome-title">欢迎回来</h2>
        <p class="welcome-desc">青灯黄卷，诗书传家。今日又是文墨飘香的一天。</p>
      </div>
      <div class="welcome-seal">
        <svg viewBox="0 0 48 48" width="48" height="48">
          <circle cx="24" cy="24" r="20" fill="none" stroke="var(--vermillion)" stroke-width="1.5" opacity="0.5"/>
          <text x="24" y="29" text-anchor="middle" fill="var(--vermillion)"
            font-size="16" font-family="serif" opacity="0.7">诗</text>
        </svg>
      </div>
    </div>

    <!-- Stats Grid -->
    <div class="stats-grid stagger">
      <div class="stat-card" v-for="item in stats" :key="item.label">
        <div class="stat-label">{{ item.label }}</div>
        <div class="stat-value" ref="statValues">{{ item.value.toLocaleString() }}</div>
      </div>
    </div>

    <!-- Quick info -->
    <div class="info-cards" style="margin-top: 24px;">
      <div class="card">
        <div class="card-title" style="font-size: var(--text-lg); margin-bottom: 16px;">快捷操作</div>
        <div style="display: flex; gap: 12px; flex-wrap: wrap;">
          <router-link to="/poets" class="btn">诗人管理</router-link>
          <router-link to="/poems" class="btn">诗词管理</router-link>
          <router-link to="/import" class="btn">批量导入</router-link>
          <router-link to="/export" class="btn">数据导出</router-link>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import api from '../api'

interface Stat {
  label: string
  value: number
}

const stats = ref<Stat[]>([])

onMounted(async () => {
  try {
    const res = await api.get('/dashboard')
    const d = res.data
    stats.value = [
      { label: '诗词总数', value: d.total_poems },
      { label: '诗人总数', value: d.total_poets },
      { label: '用户总数', value: d.total_users },
      { label: '学习记录', value: d.total_records },
    ]
  } catch {
    // handled by interceptor
  }
})
</script>

<style scoped>
/* ===== Welcome Banner ===== */
.welcome-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 28px 32px;
  margin-bottom: 28px;
  background: var(--ink-surface);
  border: 1px solid var(--ink-border);
  border-radius: var(--radius-lg);
  animation: slideUp 0.4s var(--ease-out-expo);
}

.welcome-title {
  font-size: var(--text-2xl);
  margin-bottom: 6px;
}

.welcome-desc {
  color: var(--paper-dim);
  font-size: var(--text-sm);
}

.welcome-seal {
  opacity: 0.6;
}

/* ===== Stats Grid ===== */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 8px;
}

@media (max-width: 1024px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 640px) {
  .stats-grid {
    grid-template-columns: 1fr;
  }
}

/* ===== Quick Actions ===== */
.info-cards .btn {
  font-size: var(--text-sm);
  padding: 8px 16px;
}
</style>
