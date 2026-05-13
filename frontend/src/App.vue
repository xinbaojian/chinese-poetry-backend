<template>
  <router-view v-slot="{ Component, route }">
    <transition :name="transitionName(route)" mode="out-in">
      <component :is="Component" :key="route.path" />
    </transition>
  </router-view>
</template>

<script setup lang="ts">
function transitionName(route: { meta: Record<string, unknown> }): string {
  const t = route.meta.transition
  return typeof t === 'string' ? t : 'page'
}
</script>
