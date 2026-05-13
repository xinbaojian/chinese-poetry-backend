<template>
  <div>
    <h2 style="margin-bottom: 20px;">数据导出</h2>

    <el-card>
      <el-form :inline="true">
        <el-form-item label="格式">
          <el-radio-group v-model="format">
            <el-radio value="csv">CSV</el-radio>
            <el-radio value="json">JSON</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item label="用户">
          <el-select v-model="userId" placeholder="全部用户" clearable style="width: 200px;">
            <el-option v-for="u in users" :key="u.id" :label="u.username" :value="u.id" />
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleDownload">下载</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import api from '../api'

interface UserItem {
  id: number
  username: string
}

const format = ref('csv')
const userId = ref<number | ''>('')
const users = ref<UserItem[]>([])

onMounted(async () => {
  const res = await api.get('/export/users')
  users.value = res.data.users
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
    ElMessage.success('下载开始')
  } catch {
    // handled by interceptor
  }
}
</script>
