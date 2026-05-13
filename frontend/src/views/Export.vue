<template>
  <div>
    <div class="page-header">
      <h2>数据导出</h2>
    </div>

    <div class="card">
      <div class="card-title" style="font-size: var(--text-lg); margin-bottom: 20px;">导出学习记录</div>
      <div class="filters-bar">
        <div class="filter-item">
          <span class="filter-label">导出格式</span>
          <div style="display: flex; gap: 8px;">
            <button
              class="btn btn-sm"
              :class="{ 'btn-primary': format === 'csv' }"
              @click="format = 'csv'"
            >CSV</button>
            <button
              class="btn btn-sm"
              :class="{ 'btn-primary': format === 'json' }"
              @click="format = 'json'"
            >JSON</button>
          </div>
        </div>
        <div class="filter-item">
          <span class="filter-label">用户</span>
          <select v-model="userId" class="input select" style="width: 200px;">
            <option value="">全部用户</option>
            <option v-for="u in users" :key="u.id" :value="u.id">{{ u.username }}</option>
          </select>
        </div>
        <div class="filter-item" style="align-self: flex-end;">
          <button class="btn btn-primary" @click="handleDownload">下载</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import api from '../api'
import { toast } from '../utils/toast'

interface UserItem {
  id: number
  username: string
}

const format = ref('csv')
const userId = ref<number | ''>('')
const users = ref<UserItem[]>([])

onMounted(async () => {
  try {
    const res = await api.get('/export/users')
    users.value = res.data.users
  } catch { /* handled */ }
})

async function handleDownload() {
  const params: any = { format: format.value }
  if (userId.value) params.user_id = userId.value
  try {
    const res = await api.get('/export/download', {
      params,
      responseType: 'blob',
    })
    const url = window.URL.createObjectURL(new Blob([res.data]))
    const a = document.createElement('a')
    a.href = url
    a.download = `learning_data.${format.value}`
    a.click()
    window.URL.revokeObjectURL(url)
    toast.success('下载开始')
  } catch {
    // handled by interceptor
  }
}
</script>

<style scoped>
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.page-header h2 {
  font-size: var(--text-xl);
}
</style>
