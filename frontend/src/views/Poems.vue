<template>
  <div>
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
      <h2 style="margin: 0;">诗词管理</h2>
      <el-button type="primary" @click="openCreate">添加诗词</el-button>
    </div>

    <!-- Filters -->
    <el-form :inline="true" :model="filters" style="margin-bottom: 12px;">
      <el-form-item label="搜索">
        <el-input v-model="filters.keyword" placeholder="标题" clearable @keyup.enter="search" style="width: 160px;" />
      </el-form-item>
      <el-form-item label="朝代">
        <el-select v-model="filters.dynasty" placeholder="全部" clearable style="width: 120px;">
          <el-option v-for="d in options.dynasties" :key="d" :label="d" :value="d" />
        </el-select>
      </el-form-item>
      <el-form-item label="分类">
        <el-select v-model="filters.category" placeholder="全部" clearable style="width: 140px;">
          <el-option v-for="c in options.categories" :key="c" :label="c" :value="c" />
        </el-select>
      </el-form-item>
      <el-form-item label="年级">
        <el-select v-model="filters.grade" placeholder="全部" clearable style="width: 100px;">
          <el-option v-for="g in options.grades" :key="g" :label="g" :value="g.toString()" />
        </el-select>
      </el-form-item>
      <el-form-item>
        <el-button type="primary" @click="search">搜索</el-button>
      </el-form-item>
    </el-form>

    <!-- Table -->
    <el-table :data="poems" v-loading="loading" stripe>
      <el-table-column prop="id" label="ID" width="70" />
      <el-table-column prop="title" label="标题" width="160" />
      <el-table-column prop="poet_name" label="诗人" width="100" />
      <el-table-column prop="dynasty" label="朝代" width="80" />
      <el-table-column prop="category" label="分类" width="120" />
      <el-table-column prop="grade" label="年级" width="70" />
      <el-table-column label="操作" width="160" fixed="right">
        <template #default="{ row }">
          <el-button size="small" @click="openEdit(row.id)">编辑</el-button>
          <el-button size="small" type="danger" @click="handleDelete(row.id)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <div style="display: flex; justify-content: center; margin-top: 16px;">
      <el-pagination
        v-model:current-page="page"
        :page-size="20"
        :total="total"
        layout="prev, pager, next"
        @current-change="fetchData"
      />
    </div>

    <!-- Create / Edit Dialog -->
    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑诗词' : '添加诗词'" width="640px" @close="resetForm">
      <el-form :model="form" label-width="80px">
        <el-form-item label="标题">
          <el-input v-model="form.title" placeholder="诗词标题" />
        </el-form-item>
        <el-form-item label="诗人">
          <el-autocomplete
            v-model="form.poet_name"
            :fetch-suggestions="searchPoets"
            placeholder="输入诗人名称搜索"
            value-key="name"
            style="width: 100%;"
            @select="onPoetSelect"
            clearable
          />
          <div style="font-size: 12px; color: #94a3b8; margin-top: 4px;">
            从列表选择已有诗人，或输入新名字后提交自动创建
          </div>
        </el-form-item>
        <el-form-item label="朝代">
          <el-input v-model="form.dynasty" placeholder="如：唐、宋（选择诗人后自动填充）" />
        </el-form-item>
        <el-form-item label="分类">
          <el-input v-model="form.category" placeholder="如：五言绝句、七言律诗" />
        </el-form-item>
        <el-form-item label="年级">
          <el-input-number v-model="form.grade" :min="0" :max="9" />
        </el-form-item>
        <el-form-item label="内容">
          <el-input
            v-model="form.content"
            type="textarea"
            :rows="6"
            placeholder="每行一句诗词"
          />
        </el-form-item>
        <el-form-item label="译文">
          <el-input
            v-model="form.translation"
            type="textarea"
            :rows="4"
            placeholder="可选"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSave" :loading="saving">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import api from '../api'

interface PoemItem {
  id: number
  title: string
  poet_id: number
  poet_name: string
  dynasty: string
  category: string
  grade: number
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

async function openCreate() {
  isEdit.value = false
  editId.value = 0
  dialogVisible.value = true
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
    form.category = p.category
    form.grade = p.grade
    form.content = (p.content as string[]).join('\n')
    form.translation = p.translation || ''
    dialogVisible.value = true
  } catch { /* handled */ }
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
}

async function searchPoets(query: string, cb: (results: PoetSuggestion[]) => void) {
  if (!query) {
    cb([])
    return
  }
  const res = await api.get('/poems/poets')
  const results = (res.data.poets as PoetSuggestion[]).filter((p: PoetSuggestion) =>
    p.name.includes(query)
  ).slice(0, 20)
  cb(results)
}

function onPoetSelect(item: PoetSuggestion) {
  form.poet_id = item.id
  if (!form.dynasty) {
    form.dynasty = item.dynasty
  }
}

async function handleSave() {
  if (!form.title) {
    ElMessage.warning('标题不能为空')
    return
  }
  if (!form.content.trim()) {
    ElMessage.warning('内容不能为空')
    return
  }
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
    if (form.poet_id && form.poet_id > 0) {
      payload.poet_id = form.poet_id
    }
    if (isEdit.value) {
      await api.put(`/poems/${editId.value}`, payload)
      ElMessage.success('更新成功')
    } else {
      await api.post('/poems', payload)
      ElMessage.success('创建成功')
    }
    dialogVisible.value = false
    fetchData()
  } finally {
    saving.value = false
  }
}

async function handleDelete(id: number) {
  try {
    await ElMessageBox.confirm('确定删除该诗词？', '确认', { type: 'warning' })
    await api.delete(`/poems/${id}`)
    ElMessage.success('删除成功')
    fetchData()
  } catch { /* cancelled */ }
}

onMounted(() => {
  fetchData()
  fetchOptions()
})
</script>
