<template>
  <div>
    <div class="page-header">
      <h2>诗人管理</h2>
      <button class="btn btn-primary" @click="openCreate">+ 添加诗人</button>
    </div>

    <!-- Filters -->
    <div class="filters-bar">
      <div class="filter-item">
        <span class="filter-label">搜索</span>
        <input
          v-model="filters.keyword"
          class="input"
          placeholder="诗人名称"
          style="width: 180px;"
          @keyup.enter="search"
        />
      </div>
      <div class="filter-item">
        <span class="filter-label">朝代</span>
        <select v-model="filters.dynasty" class="input select" style="width: 140px;" @change="search">
          <option value="">全部</option>
          <option v-for="d in dynasties" :key="d" :value="d">{{ d }}</option>
        </select>
      </div>
      <div class="filter-item" style="align-self: flex-end;">
        <button class="btn btn-primary btn-sm" @click="search">搜索</button>
      </div>
    </div>

    <!-- Table -->
    <div class="table-wrap">
      <table class="table">
        <thead>
          <tr>
            <th class="col-id">ID</th>
            <th>姓名</th>
            <th>朝代</th>
            <th>创建时间</th>
            <th style="width: 140px;">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="loading && poets.length === 0">
            <td colspan="5">
              <div style="padding: 32px; text-align: center; color: var(--paper-dim);">加载中…</div>
            </td>
          </tr>
          <tr v-else-if="poets.length === 0">
            <td colspan="5">
              <div style="padding: 32px; text-align: center; color: var(--paper-dim);">暂无数据</div>
            </td>
          </tr>
          <tr v-for="poet in poets" :key="poet.id">
            <td class="col-id">{{ poet.id }}</td>
            <td>{{ poet.name }}</td>
            <td>{{ poet.dynasty }}</td>
            <td>{{ poet.created_at }}</td>
            <td>
              <div style="display: flex; gap: 6px;">
                <button class="btn btn-sm" @click="openEdit(poet)">编辑</button>
                <button class="btn btn-sm btn-danger" @click="handleDelete(poet.id)">删除</button>
              </div>
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

    <!-- Dialog -->
    <div v-if="dialogVisible" class="dialog-overlay" @click.self="dialogVisible = false">
      <div class="dialog" style="max-width: 420px;">
        <div class="dialog-header">
          <span class="dialog-title">{{ isEdit ? '编辑诗人' : '添加诗人' }}</span>
          <button class="btn btn-icon btn-ghost" @click="dialogVisible = false">✕</button>
        </div>
        <div class="dialog-body">
          <div class="form-group">
            <label class="form-label">姓名</label>
            <input v-model="form.name" class="input" placeholder="诗人姓名" />
          </div>
          <div class="form-group">
            <label class="form-label">朝代</label>
            <input v-model="form.dynasty" class="input" placeholder="如：唐、宋" />
          </div>
        </div>
        <div class="dialog-footer">
          <button class="btn" @click="dialogVisible = false">取消</button>
          <button class="btn btn-primary" @click="handleSave" :disabled="saving">
            {{ saving ? '保存中…' : '保存' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import api from '../api'
import { toast } from '../utils/toast'
import { confirmDialog } from '../utils/confirm'

interface Poet {
  id: number
  name: string
  dynasty: string
  created_at: string
}

const poets = ref<Poet[]>([])
const dynasties = ref<string[]>([])
const loading = ref(false)
const saving = ref(false)
const total = ref(0)
const page = ref(1)
const dialogVisible = ref(false)
const isEdit = ref(false)
const editId = ref(0)

const filters = reactive({ keyword: '', dynasty: '' })
const form = reactive({ name: '', dynasty: '' })

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
    const res = await api.get('/poets', { params: { ...filters, page: page.value } })
    poets.value = res.data.poets
    total.value = res.data.total
  } finally {
    loading.value = false
  }
}

async function fetchDynasties() {
  const res = await api.get('/poets/dynasties')
  dynasties.value = res.data.dynasties
}

function search() {
  page.value = 1
  fetchData()
}

function goPage(p: number) {
  if (p < 1 || p > totalPages.value) return
  page.value = p
  fetchData()
}

function openCreate() {
  isEdit.value = false
  editId.value = 0
  form.name = ''
  form.dynasty = ''
  dialogVisible.value = true
}

function openEdit(row: Poet) {
  isEdit.value = true
  editId.value = row.id
  form.name = row.name
  form.dynasty = row.dynasty
  dialogVisible.value = true
}

async function handleSave() {
  if (!form.name || !form.dynasty) {
    toast.warning('姓名和朝代不能为空')
    return
  }
  saving.value = true
  try {
    if (isEdit.value) {
      await api.put(`/poets/${editId.value}`, form)
      toast.success('更新成功')
    } else {
      await api.post('/poets', form)
      toast.success('创建成功')
    }
    dialogVisible.value = false
    fetchData()
  } finally {
    saving.value = false
  }
}

async function handleDelete(id: number) {
  const ok = await confirmDialog('确定删除该诗人？')
  if (!ok) return
  await api.delete(`/poets/${id}`)
  toast.success('删除成功')
  fetchData()
}

onMounted(() => {
  fetchData()
  fetchDynasties()
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
</style>
