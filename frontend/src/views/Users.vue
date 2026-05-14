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
            <th style="width: 220px;">操作</th>
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
              <div style="display: flex; gap: 6px;">
                <button class="btn btn-sm" @click="openProgress(user)">学习进度</button>
                <button class="btn btn-sm" @click="handleResetPassword(user.id, user.username)">重置密码</button>
                <button class="btn btn-sm btn-danger" @click="handleDelete(user.id)">删除</button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Pagination -->
    <div class="pagination" v-if="total > perPage">
      <div class="pagination-left">
        <select v-model.number="perPage" class="input select per-page-select" @change="changePerPage">
          <option :value="10">10 条/页</option>
          <option :value="20">20 条/页</option>
          <option :value="50">50 条/页</option>
        </select>
      </div>
      <div class="pagination-right">
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

    <!-- Progress Dialog -->
    <div v-if="progressVisible" class="dialog-overlay" @click.self="progressVisible = false">
      <div class="dialog" style="max-width: 720px;">
        <div class="dialog-header">
          <span class="dialog-title">学习进度 — {{ progressUsername }}</span>
          <button class="btn btn-icon btn-ghost" @click="progressVisible = false">✕</button>
        </div>
        <div class="dialog-body" style="padding-bottom: 0;">
          <div v-if="progressLoading" style="padding: 32px; text-align: center; color: var(--paper-dim);">加载中…</div>
          <div v-else-if="progressRecords.length === 0" style="padding: 32px; text-align: center; color: var(--paper-dim);">暂无学习记录</div>
          <table v-else class="table table-compact">
            <thead>
              <tr>
                <th>诗词</th>
                <th>诗人</th>
                <th>掌握程度</th>
                <th>复习次数</th>
                <th>下次复习</th>
                <th>最近更新</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="r in progressRecords" :key="r.id">
                <td>{{ r.poem_title }}</td>
                <td>{{ r.poet_name }}</td>
                <td>
                  <span class="mastery-badge" :class="'mastery-' + r.mastery_level">
                    {{ masteryLabel(r.mastery_level) }}
                  </span>
                </td>
                <td>{{ r.review_count }}</td>
                <td>{{ r.next_review_date || '—' }}</td>
                <td>{{ r.updated_at }}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <div class="dialog-footer">
          <button class="btn" @click="progressVisible = false">关闭</button>
        </div>
      </div>
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

interface ProgressRecord {
  id: number
  poem_id: number
  poem_title: string
  poet_name: string
  mastery_level: string
  review_count: number
  next_review_date: string | null
  updated_at: string
}

const users = ref<User[]>([])
const loading = ref(false)
const total = ref(0)
const page = ref(1)
const perPage = ref(10)

// Progress dialog state
const progressVisible = ref(false)
const progressLoading = ref(false)
const progressUsername = ref('')
const progressRecords = ref<ProgressRecord[]>([])

const masteryMap: Record<string, string> = {
  proficient: '精通',
  fair: '一般',
  weak: '薄弱',
}

function masteryLabel(level: string): string {
  return masteryMap[level] || level
}

async function openProgress(user: User) {
  progressUsername.value = user.username
  progressVisible.value = true
  progressLoading.value = true
  progressRecords.value = []
  try {
    const res = await api.get(`/users/${user.id}/progress`)
    progressRecords.value = res.data
  } catch {
    progressVisible.value = false
  } finally {
    progressLoading.value = false
  }
}

const totalPages = computed(() => Math.ceil(total.value / perPage.value))
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
    const res = await api.get('/users', { params: { page: page.value, per_page: perPage.value } })
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

function changePerPage() {
  page.value = 1
  fetchData()
}

async function handleResetPassword(id: number, username: string) {
  const ok = await confirmDialog(`确定将用户「${username}」的密码重置为默认密码？`)
  if (!ok) return
  await api.put(`/users/${id}/reset-password`)
  toast.success('密码已重置为默认密码')
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

.pagination {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.pagination-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pagination-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

.per-page-select {
  width: auto;
  padding: 4px 8px;
  font-size: var(--text-xs);
}

.mastery-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  font-size: var(--text-xs);
}

.mastery-proficient {
  background: rgba(76, 175, 80, 0.15);
  color: #4caf50;
  border: 1px solid rgba(76, 175, 80, 0.3);
}

.mastery-fair {
  background: rgba(255, 152, 0, 0.15);
  color: #ff9800;
  border: 1px solid rgba(255, 152, 0, 0.3);
}

.mastery-weak {
  background: rgba(194, 54, 62, 0.15);
  color: var(--vermillion);
  border: 1px solid rgba(194, 54, 62, 0.3);
}

.table-compact {
  font-size: var(--text-sm);
}

.table-compact td,
.table-compact th {
  padding: 6px 8px;
}
</style>
