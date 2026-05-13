<template>
  <div>
    <div class="page-header">
      <h2>批量导入</h2>
    </div>

    <div class="card" style="margin-bottom: 24px;">
      <div class="card-title" style="font-size: var(--text-lg); margin-bottom: 20px;">导入 JSON 数据</div>
      <div class="form-group">
        <label class="form-label">默认朝代</label>
        <input
          v-model="form.dynasty"
          class="input"
          placeholder="缺少朝代时使用此值，默认：唐"
          style="max-width: 240px;"
        />
      </div>
      <div class="form-group">
        <label class="form-label">JSON 数据</label>
        <textarea
          v-model="form.json_data"
          class="input textarea"
          rows="14"
          placeholder='[
  {
    "title": "静夜思",
    "author": "李白",
    "dynasty": "唐",
    "paragraphs": ["床前明月光", "疑是地上霜", "举头望明月", "低头思故乡"]
  }
]'
        ></textarea>
      </div>
      <button
        class="btn btn-primary"
        @click="handleImport"
        :disabled="importing"
      >
        {{ importing ? '导入中…' : '开始导入' }}
      </button>
    </div>

    <!-- Results -->
    <div v-if="results" class="card">
      <div class="card-title" style="font-size: var(--text-lg); margin-bottom: 20px;">导入结果</div>
      <div class="result-grid">
        <div class="stat-card">
          <div class="stat-label">成功导入</div>
          <div class="stat-value" style="color: var(--jade); font-size: var(--text-2xl);">{{ results.imported }}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">跳过（重复）</div>
          <div class="stat-value" style="color: var(--gold); font-size: var(--text-2xl);">{{ results.skipped }}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">失败</div>
          <div class="stat-value" style="color: var(--vermillion); font-size: var(--text-2xl);">{{ results.failed }}</div>
        </div>
      </div>
      <div v-if="results.errors.length > 0" style="margin-top: 20px;">
        <h4 style="color: var(--vermillion); font-family: 'Noto Serif SC', serif; font-weight: 600; margin-bottom: 8px;">错误详情</h4>
        <ul style="color: var(--paper-dim); font-size: var(--text-sm); padding-left: 20px;">
          <li v-for="(e, i) in results.errors" :key="i" style="margin-bottom: 4px;">{{ e }}</li>
        </ul>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import api from '../api'
import { toast } from '../utils/toast'

interface ImportResults {
  imported: number
  skipped: number
  failed: number
  errors: string[]
}

const form = reactive({ json_data: '', dynasty: '' })
const importing = ref(false)
const results = ref<ImportResults | null>(null)

async function handleImport() {
  if (!form.json_data.trim()) {
    toast.warning('请粘贴 JSON 数据')
    return
  }
  importing.value = true
  try {
    const res = await api.post('/import', form)
    results.value = res.data
    toast.success('导入完成')
  } finally {
    importing.value = false
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

.result-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}
</style>
