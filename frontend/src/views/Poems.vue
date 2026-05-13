<template>
  <div>
    <div class="page-header">
      <h2>诗词管理</h2>
      <button class="btn btn-primary" @click="openCreate">+ 添加诗词</button>
    </div>

    <div class="filters-bar">
      <div class="filter-item">
        <span class="filter-label">搜索</span>
        <input v-model="filters.keyword" class="input" placeholder="标题关键词" style="width: 160px;" @keyup.enter="search" />
      </div>
      <div class="filter-item">
        <span class="filter-label">朝代</span>
        <select v-model="filters.dynasty" class="input select" style="width: 120px;" @change="search">
          <option value="">全部</option>
          <option v-for="d in options.dynasties" :key="d" :value="d">{{ d }}</option>
        </select>
      </div>
      <div class="filter-item">
        <span class="filter-label">分类</span>
        <select v-model="filters.category" class="input select" style="width: 140px;" @change="search">
          <option value="">全部</option>
          <option v-for="c in options.categories" :key="c" :value="c">{{ c }}</option>
        </select>
      </div>
      <div class="filter-item">
        <span class="filter-label">年级</span>
        <select v-model="filters.grade" class="input select" style="width: 100px;" @change="search">
          <option value="">全部</option>
          <option v-for="g in options.grades" :key="g" :value="g">{{ g }}</option>
        </select>
      </div>
      <div class="filter-item" style="align-self: flex-end;">
        <button class="btn btn-primary btn-sm" @click="search">搜索</button>
      </div>
    </div>

    <div class="table-wrap">
      <table class="table">
        <thead>
          <tr>
            <th class="col-id">ID</th>
            <th>标题</th>
            <th>诗人</th>
            <th>朝代</th>
            <th>分类</th>
            <th>年级</th>
            <th style="width: 180px;">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="loading && poems.length === 0">
            <td colspan="7">
              <div style="padding: 32px; text-align: center; color: var(--paper-dim);">加载中…</div>
            </td>
          </tr>
          <tr v-else-if="poems.length === 0">
            <td colspan="7">
              <div style="padding: 32px; text-align: center; color: var(--paper-dim);">暂无数据</div>
            </td>
          </tr>
          <tr v-for="poem in poems" :key="poem.id">
            <td class="col-id">{{ poem.id }}</td>
            <td>{{ poem.title }}</td>
            <td>{{ poem.poet_name }}</td>
            <td>{{ poem.dynasty }}</td>
            <td>{{ poem.category || '—' }}</td>
            <td>{{ poem.grade || '—' }}</td>
            <td>
              <div style="display: flex; gap: 6px;">
                <button class="btn btn-sm" @click="openDetail(poem.id)">查看</button>
                <button class="btn btn-sm" @click="openEdit(poem.id)">编辑</button>
                <button class="btn btn-sm btn-danger" @click="handleDelete(poem.id)">删除</button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="pagination" v-if="total > 20">
      <button class="btn btn-sm" :disabled="page <= 1" @click="goPage(page - 1)">上一页</button>
      <button v-for="p in pages" :key="p" class="btn btn-sm" :class="{ active: p === page }" @click="goPage(p)">{{ p }}</button>
      <button class="btn btn-sm" :disabled="page >= totalPages" @click="goPage(page + 1)">下一页</button>
    </div>

    <!-- Detail View -->
    <div v-if="detailVisible" class="dialog-overlay" @click.self="detailVisible = false">
      <div class="detail-dialog">
        <button class="detail-close btn btn-icon btn-ghost" @click="detailVisible = false">✕</button>
        <div v-if="detailLoading" style="padding: 80px 0; text-align: center; color: var(--paper-dim);">加载中…</div>
        <template v-else-if="detail">
          <h2 class="detail-title">{{ detail.title }}</h2>
          <div class="detail-header">
            <div class="detail-meta">
              <span class="detail-dynasty">{{ detail.dynasty }}</span>
              <span class="detail-dot">·</span>
              <span class="detail-poet">{{ detail.poet_name }}</span>
            </div>
          </div>
          <div class="detail-divider"></div>
          <div class="detail-content">
            <p v-for="(line, i) in detail.content" :key="i" class="detail-line">{{ line }}</p>
          </div>
          <template v-if="detail.translation">
            <div class="detail-divider"></div>
            <div class="detail-section-label">译文</div>
            <p class="detail-translation">{{ detail.translation }}</p>
          </template>
          <div class="detail-footer">
            <span v-if="detail.category" class="detail-tag">{{ detail.category }}</span>
            <span v-if="detail.grade" class="detail-tag">年级 {{ detail.grade }}</span>
          </div>
        </template>
      </div>
    </div>

    <!-- Edit / Create Dialog -->
    <div v-if="dialogVisible" class="dialog-overlay" @click.self="dialogVisible = false">
      <div class="dialog" style="max-width: 600px;">
        <div class="dialog-header">
          <span class="dialog-title">{{ isEdit ? '编辑诗词' : '添加诗词' }}</span>
          <button class="btn btn-icon btn-ghost" @click="dialogVisible = false">✕</button>
        </div>
        <div class="dialog-body">
          <div class="form-group">
            <label class="form-label">标题</label>
            <input v-model="form.title" class="input" placeholder="诗词标题" />
          </div>
          <div class="form-group" style="position: relative;">
            <label class="form-label">诗人</label>
            <input v-model="form.poet_name" class="input" placeholder="输入诗人名称" @input="onPoetNameInput" />
            <div class="form-hint">从列表选择已有诗人，或输入新名字后提交自动创建</div>
            <div v-if="poetSuggestions.length > 0" class="suggestions-dropdown">
              <div v-for="s in poetSuggestions" :key="s.id" class="suggestion-item" @click="selectPoet(s)">
                <span>{{ s.name }}</span>
                <span class="suggestion-dynasty">{{ s.dynasty }}</span>
              </div>
            </div>
          </div>
          <div class="form-group">
            <label class="form-label">朝代</label>
            <input v-model="form.dynasty" class="input" placeholder="如：唐、宋" />
          </div>
          <div class="form-group">
            <label class="form-label">分类</label>
            <input v-model="form.category" class="input" placeholder="如：五言绝句、七言律诗" />
          </div>
          <div class="form-group">
            <label class="form-label">年级</label>
            <input v-model.number="form.grade" class="input" type="number" min="0" max="9" style="width: 100px;" />
          </div>
          <div class="form-group">
            <label class="form-label">内容</label>
            <textarea v-model="form.content" class="input textarea" rows="6" placeholder="每行一句诗词"></textarea>
          </div>
          <div class="form-group">
            <label class="form-label">译文</label>
            <textarea v-model="form.translation" class="input textarea" rows="4" placeholder="可选"></textarea>
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

interface PoemItem {
  id: number
  title: string
  poet_id: number
  poet_name: string
  dynasty: string
  category: string
  grade: number
}

interface PoemDetail {
  id: number
  title: string
  poet_id: number
  poet_name: string
  dynasty: string
  category: string
  grade: number
  content: string[]
  translation: string | null
}

interface PoetSuggestion {
  id: number
  name: string
  dynasty: string
}

const poems = ref<PoemItem[]>([])
const loading = ref(false)
const saving = ref(false)
const total = ref(0)
const page = ref(1)
const dialogVisible = ref(false)
const isEdit = ref(false)
const editId = ref(0)
const poetSuggestions = ref<PoetSuggestion[]>([])
let allPoets: PoetSuggestion[] = []

// Detail state
const detailVisible = ref(false)
const detailLoading = ref(false)
const detail = ref<PoemDetail | null>(null)

const filters = reactive({ keyword: '', dynasty: '', category: '', grade: '' })
const form = reactive({
  title: '',
  poet_id: 0,
  poet_name: '',
  dynasty: '',
  category: '',
  grade: 0,
  content: '',
  translation: '',
})

const options = reactive({ dynasties: [] as string[], categories: [] as string[], grades: [] as number[] })

const totalPages = computed(() => Math.ceil(total.value / 20))
const pages = computed(() => {
  const p: number[] = []
  const t = totalPages.value
  if (t <= 7) { for (let i = 1; i <= t; i++) p.push(i) }
  else {
    p.push(1)
    const start = Math.max(2, page.value - 2)
    const end = Math.min(t - 1, page.value + 2)
    if (start > 2) p.push(-1)
    for (let i = start; i <= end; i++) p.push(i)
    if (end < t - 1) p.push(-1)
    p.push(t)
  }
  return p
})

async function fetchData() {
  loading.value = true
  try {
    const res = await api.get('/poems', { params: { ...filters, page: page.value } })
    poems.value = res.data.poems
    total.value = res.data.total
  } finally {
    loading.value = false
  }
}

async function fetchOptions() {
  const res = await api.get('/poems/filter-options')
  options.dynasties = res.data.dynasties
  options.categories = res.data.categories
  options.grades = res.data.grades
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
  resetForm()
  dialogVisible.value = true
}

async function openDetail(id: number) {
  detailVisible.value = true
  detailLoading.value = true
  detail.value = null
  try {
    const res = await api.get(`/poems/${id}`)
    detail.value = res.data
  } catch {
    detailVisible.value = false
  } finally {
    detailLoading.value = false
  }
}

async function openEdit(id: number) {
  isEdit.value = true
  editId.value = id
  try {
    const res = await api.get(`/poems/${id}`)
    const p = res.data
    form.title = p.title
    form.poet_id = p.poet_id
    form.poet_name = p.poet_name
    form.dynasty = p.dynasty
    form.category = p.category || ''
    form.grade = p.grade
    form.content = Array.isArray(p.content) ? p.content.join('\n') : p.content
    form.translation = p.translation || ''
    dialogVisible.value = true
  } catch { /* handled by interceptor */ }
}

function resetForm() {
  form.title = ''
  form.poet_id = 0
  form.poet_name = ''
  form.dynasty = ''
  form.category = ''
  form.grade = 0
  form.content = ''
  form.translation = ''
  poetSuggestions.value = []
}

function onPoetNameInput() {
  if (!form.poet_name) {
    poetSuggestions.value = []
    form.poet_id = 0
    return
  }
  poetSuggestions.value = allPoets
    .filter(p => p.name.includes(form.poet_name))
    .slice(0, 8)
}

function selectPoet(item: PoetSuggestion) {
  form.poet_id = item.id
  form.poet_name = item.name
  if (!form.dynasty) form.dynasty = item.dynasty
  poetSuggestions.value = []
}

async function handleSave() {
  if (!form.title) { toast.warning('标题不能为空'); return }
  if (!form.content.trim()) { toast.warning('内容不能为空'); return }
  saving.value = true
  try {
    const payload: any = {
      title: form.title,
      poet_name: form.poet_name,
      dynasty: form.dynasty,
      category: form.category || null,
      grade: form.grade,
      content: form.content,
      translation: form.translation || null,
    }
    if (form.poet_id > 0) payload.poet_id = form.poet_id
    if (isEdit.value) {
      await api.put(`/poems/${editId.value}`, payload)
      toast.success('更新成功')
    } else {
      await api.post('/poems', payload)
      toast.success('创建成功')
    }
    dialogVisible.value = false
    fetchData()
  } finally {
    saving.value = false
  }
}

async function handleDelete(id: number) {
  const ok = await confirmDialog('确定删除该诗词？')
  if (!ok) return
  await api.delete(`/poems/${id}`)
  toast.success('删除成功')
  fetchData()
}

onMounted(async () => {
  await Promise.all([fetchData(), fetchOptions()])
  try {
    const res = await api.get('/poems/poets')
    allPoets = res.data.poets
  } catch { /* non-critical */ }
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

/* ===== Suggestions Dropdown ===== */
.suggestions-dropdown {
  position: absolute;
  z-index: 10;
  background: var(--ink-raised);
  border: 1px solid var(--ink-border);
  border-radius: var(--radius-md);
  margin-top: 4px;
  max-height: 200px;
  overflow-y: auto;
  width: 100%;
  box-shadow: var(--shadow-card);
}

.suggestion-item {
  padding: 8px 12px;
  cursor: pointer;
  display: flex;
  justify-content: space-between;
  transition: background var(--duration-fast);
}

.suggestion-item:hover {
  background: var(--ink-overlay);
}

.suggestion-dynasty {
  color: var(--paper-dim);
  font-size: var(--text-xs);
}

/* ===== Detail Dialog ===== */
.detail-dialog {
  position: relative;
  background: var(--ink-surface);
  border: 1px solid var(--ink-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-dialog);
  width: 90%;
  max-width: 520px;
  max-height: 85vh;
  overflow-y: auto;
  padding: 48px 48px 36px;
  animation: inkBleed 0.3s var(--ease-out-expo);
}

.detail-close {
  position: absolute;
  top: 16px;
  right: 16px;
}

.detail-header {
  text-align: center;
  margin-bottom: 8px;
}

.detail-meta {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: var(--text-sm);
  color: var(--paper-dim);
}

.detail-poet {
  color: var(--gold);
}

.detail-dot {
  color: var(--paper-faint);
}

.detail-title {
  text-align: center;
  font-size: var(--text-2xl);
  color: var(--paper-bright);
  letter-spacing: 0.12em;
  margin-bottom: 0;
}

.detail-divider {
  height: 1px;
  background: linear-gradient(to right, transparent, var(--ink-border) 30%, var(--ink-border) 70%, transparent);
  margin: 24px 0;
}

.detail-content {
  text-align: center;
}

.detail-line {
  font-size: var(--text-lg);
  color: var(--paper-bright);
  line-height: 2.2;
  margin: 0;
  letter-spacing: 0.08em;
}

.detail-section-label {
  font-size: var(--text-xs);
  color: var(--paper-faint);
  text-align: center;
  letter-spacing: 0.15em;
  margin-bottom: 12px;
}

.detail-translation {
  font-size: var(--text-base);
  color: var(--paper-dim);
  line-height: 1.9;
  text-align: center;
  margin: 0;
}

.detail-footer {
  display: flex;
  justify-content: center;
  gap: 10px;
  margin-top: 8px;
}

.detail-tag {
  display: inline-block;
  padding: 3px 12px;
  border-radius: var(--radius-sm);
  font-size: var(--text-xs);
  color: var(--paper-dim);
  border: 1px solid var(--ink-border);
}
</style>
