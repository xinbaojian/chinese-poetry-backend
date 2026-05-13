<template>
  <div>
    <h2 style="margin-bottom: 20px;">仪表盘</h2>
    <el-row :gutter="16">
      <el-col :span="6" v-for="item in stats" :key="item.label">
        <el-card>
          <el-statistic :value="item.value">
            <template #title>{{ item.label }}</template>
          </el-statistic>
        </el-card>
      </el-col>
    </el-row>
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
  const res = await api.get('/dashboard')
  const d = res.data
  stats.value = [
    { label: '诗词总数', value: d.total_poems },
    { label: '诗人总数', value: d.total_poets },
    { label: '用户总数', value: d.total_users },
    { label: '学习记录', value: d.total_records },
  ]
})
</script>
