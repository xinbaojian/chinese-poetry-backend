<template>
  <div>
    <h2 style="margin-bottom: 20px;">批量导入</h2>

    <el-card style="margin-bottom: 20px;">
      <el-form :model="form" label-width="80px">
        <el-form-item label="默认朝代">
          <el-input v-model="form.dynasty" placeholder="缺少朝代时使用此值，默认：唐" style="width: 200px;" />
        </el-form-item>
        <el-form-item label="JSON 数据">
          <el-input
            v-model="form.json_data"
            type="textarea"
            :rows="12"
            placeholder='[
  {
    "title": "静夜思",
    "author": "李白",
    "dynasty": "唐",
    "paragraphs": ["床前明月光", "疑是地上霜", "举头望明月", "低头思故乡"]
  }
]'
          />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleImport" :loading="importing">开始导入</el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- Results -->
    <el-card v-if="results !== null">
      <template #header><span>导入结果</span></template>
      <el-row :gutter="16">
        <el-col :span="8">
          <el-statistic :value="results.imported">
            <template #title>成功导入</template>
          </el-statistic>
        </el-col>
        <el-col :span="8">
          <el-statistic :value="results.skipped">
            <template #title>跳过（重复）</template>
          </el-statistic>
        </el-col>
        <el-col :span="8">
          <el-statistic :value="results.failed">
            <template #title>失败</template>
          </el-statistic>
        </el-col>
      </el-row>
      <div v-if="results.errors.length > 0" style="margin-top: 16px;">
        <h4>错误详情：</h4>
        <ul style="color: #ef4444; font-size: 13px;">
          <li v-for="(e, i) in results.errors" :key="i">{{ e }}</li>
        </ul>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { ElMessage } from 'element-plus'
import api from '../api'

interface ImportResults {
  imported: number
  skipped: number
  failed: number
  errors: string[]
}

const form = reactive({
  json_data: '',
  dynasty: '',
})

const importing = ref(false)
const results = ref<ImportResults | null>(null)

async function handleImport() {
  if (!form.json_data.trim()) {
    ElMessage.warning('请粘贴 JSON 数据')
    return
  }
  importing.value = true
  try {
    const res = await api.post('/import', form)
    results.value = res.data
  } finally {
    importing.value = false
  }
}
</script>
