<template>
  <el-container style="min-height: 100vh;">
    <el-aside width="220px" style="background-color: #1e293b;">
      <div style="padding: 18px 20px; border-bottom: 1px solid #334155;">
        <h2 style="color: #fff; font-size: 17px; margin: 0;">古诗词管理</h2>
      </div>
      <el-menu
        :default-active="activeRoute"
        router
        background-color="#1e293b"
        text-color="#cbd5e1"
        active-text-color="#fff"
        style="border-right: none;"
      >
        <el-menu-item index="/dashboard">
          <span>仪表盘</span>
        </el-menu-item>
        <el-menu-item index="/poets">
          <span>诗人管理</span>
        </el-menu-item>
        <el-menu-item index="/poems">
          <span>诗词管理</span>
        </el-menu-item>
        <el-menu-item index="/users">
          <span>用户管理</span>
        </el-menu-item>
        <el-menu-item index="/import">
          <span>批量导入</span>
        </el-menu-item>
        <el-menu-item index="/export">
          <span>数据导出</span>
        </el-menu-item>
      </el-menu>
      <div style="position: absolute; bottom: 0; width: 220px; padding: 12px 20px; border-top: 1px solid #334155;">
        <el-button text style="color: #94a3b8; width: 100%;" @click="logout">退出登录</el-button>
      </div>
    </el-aside>
    <el-main style="background: #f1f5f9; padding: 24px;">
      <router-view />
    </el-main>
  </el-container>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()
const activeRoute = computed(() => route.path)

function logout() {
  localStorage.removeItem('admin_token')
  router.push('/login')
}
</script>
