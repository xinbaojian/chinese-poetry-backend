<template>
  <div>
    <div class="page-header">
      <h2>用户管理</h2>
    </div>

    <!-- Table -->
    <div class="table-wrap">
      <table class="table">
        <thead>
          <tr>
            <th class="col-id">ID</th>
            <th>用户名</th>
            <th>角色</th>
            <th>学习记录</th>
            <th>注册时间</th>
            <th style="width: 100px;">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="loading && users.length === 0">
            <td colspan="6">
              <div style="padding: 32px; text-align: center; color: var(--paper-dim);">加载中…</div>
            </td>
          </tr>
          <tr v-else-if="users.length === 0">
            <td colspan="6">
              <div style="padding: 32px; text-align: center; color: var(--paper-dim);">暂无数据</div>
            </td>
          </tr>
          <tr v-for="user in users" :key="user.id">
            <td class="col-id">{{ user.id }}</td>
            <td>{{ user.username }}</td>
            <td>
              <span class="role-badge" :class="user.role === 'admin' ? 'role-admin' : 'role-user'">
                {{ user.role === 'admin' ? '管理员' : '用户' }}
              </span>
            </td>
            <td>{{ user.record_count }}</td>
            <td>{{ user.created_at }}</td>
            <td>
              <button class="btn btn-sm btn-danger" @click="handleDelete(user.id)">删除</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Pagination -->
    <div class="pagination" v-if="total > 20">
      <button class="btn btn-sm" :disabled="page <= 1" @click="goPage(page - 1)">上一页</button>
      <button
        v-for="p in pages" :key="p"
        class="btn btn-sm"
        :class="{ active: p === page }"
        @click="goPage(p)"
      >{{ p }}</button>
      <button class="btn btn-sm" :disabled="page >= totalPages" @click="goPage(page + 1)">下一页</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import api from '../api'
import { toast } from '../utils/toast'
import { confirmDialog } from '../utils/confirm'

interface User {
  id: number
  username: string
  role: string
  record_count: number
  created_at: string
}

const users = ref<User[]>([])
const loading = ref(false)
const total = ref(0)
const page = ref(1)

const totalPages = computed(() => Math.ceil(total.value / 20))
const pages = computed(() => {
  const t = totalPages.value
  if (t <= 7) {
    const p: number[] = []
    for (let i = 1; i <= t; i++) p.push(i)
    return p
  }
  const p: number[] = [1]
  const start = Math.max(2, page.value - 2)
  const end = Math.min(t - 1, page.value + 2)
  if (start > 2) p.push(-1)
  for (let i = start; i <= end; i++) p.push(i)
  if (end < t - 1) p.push(-1)
  p.push(t)
  return p
})

async function fetchData() {
  loading.value = true
  try {
    const res = await api.get('/users', { params: { page: page.value } })
    users.value = res.data.users
    total.value = res.data.total
  } finally {
    loading.value = false
  }
}

function goPage(p: number) {
  if (p < 1 || p > totalPages.value) return
  page.value = p
  fetchData()
}

async function handleDelete(id: number) {
  const ok = await confirmDialog('确定删除该用户及相关学习记录？')
  if (!ok) return
  await api.delete(`/users/${id}`)
  toast.success('删除成功')
  fetchData()
}

onMounted(() => {
  fetchData()
})
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

.role-badge {
  display: inline-block;
  padding: 2px 10px;
  border-radius: var(--radius-sm);
  font-family: 'Noto Serif SC', serif; font-weight: 600;
  font-size: var(--text-xs);
}

.role-admin {
  background: rgba(194, 54, 62, 0.15);
  color: var(--vermillion);
  border: 1px solid rgba(194, 54, 62, 0.3);
}

.role-user {
  background: rgba(191, 160, 96, 0.1);
  color: var(--gold);
  border: 1px solid rgba(191, 160, 96, 0.2);
}
</style>
