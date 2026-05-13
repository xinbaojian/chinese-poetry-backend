<template>
  <div>
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
      <h2 style="margin: 0;">诗人管理</h2>
      <el-button type="primary" @click="openCreate">添加诗人</el-button>
    </div>

    <!-- Filters -->
    <el-form :inline="true" :model="filters" style="margin-bottom: 12px;">
      <el-form-item label="搜索">
        <el-input v-model="filters.keyword" placeholder="诗人名称" clearable @keyup.enter="search" />
      </el-form-item>
      <el-form-item label="朝代">
        <el-select v-model="filters.dynasty" placeholder="全部" clearable style="width: 140px;">
          <el-option v-for="d in dynasties" :key="d" :label="d" :value="d" />
        </el-select>
      </el-form-item>
      <el-form-item>
        <el-button type="primary" @click="search">搜索</el-button>
      </el-form-item>
    </el-form>

    <!-- Table -->
    <el-table :data="poets" v-loading="loading" stripe>
      <el-table-column prop="id" label="ID" width="80" />
      <el-table-column prop="name" label="姓名" />
      <el-table-column prop="dynasty" label="朝代" width="100" />
      <el-table-column prop="created_at" label="创建时间" width="180" />
      <el-table-column label="操作" width="160" fixed="right">
        <template #default="{ row }">
          <el-button size="small" @click="openEdit(row)">编辑</el-button>
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
    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑诗人' : '添加诗人'" width="480px">
      <el-form :model="form" label-width="80px">
        <el-form-item label="姓名">
          <el-input v-model="form.name" placeholder="诗人姓名" />
        </el-form-item>
        <el-form-item label="朝代">
          <el-input v-model="form.dynasty" placeholder="如：唐、宋" />
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
    ElMessage.warning('姓名和朝代不能为空')
    return
  }
  saving.value = true
  try {
    if (isEdit.value) {
      await api.put(`/poets/${editId.value}`, form)
      ElMessage.success('更新成功')
    } else {
      await api.post('/poets', form)
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
    await ElMessageBox.confirm('确定删除该诗人？', '确认', { type: 'warning' })
    await api.delete(`/poets/${id}`)
    ElMessage.success('删除成功')
    fetchData()
  } catch { /* cancelled */ }
}

onMounted(() => {
  fetchData()
  fetchDynasties()
})
</script>
