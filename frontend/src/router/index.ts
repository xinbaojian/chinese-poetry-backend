import { createRouter, createWebHistory } from 'vue-router'
import AdminLayout from '../layouts/AdminLayout.vue'
import Login from '../views/Login.vue'
import Dashboard from '../views/Dashboard.vue'
import Poets from '../views/Poets.vue'
import Poems from '../views/Poems.vue'
import Users from '../views/Users.vue'
import Import from '../views/Import.vue'
import Export from '../views/Export.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      name: 'Login',
      component: Login,
    },
    {
      path: '/',
      component: AdminLayout,
      children: [
        {
          path: '',
          redirect: '/dashboard',
        },
        {
          path: 'dashboard',
          name: 'Dashboard',
          component: Dashboard,
        },
        {
          path: 'poets',
          name: 'Poets',
          component: Poets,
        },
        {
          path: 'poems',
          name: 'Poems',
          component: Poems,
        },
        {
          path: 'users',
          name: 'Users',
          component: Users,
        },
        {
          path: 'import',
          name: 'Import',
          component: Import,
        },
        {
          path: 'export',
          name: 'Export',
          component: Export,
        },
      ],
    },
  ],
})

// Auth guard
router.beforeEach((to, _from, next) => {
  const token = localStorage.getItem('admin_token')
  if (to.name !== 'Login' && !token) {
    next({ name: 'Login' })
  } else if (to.name === 'Login' && token) {
    next({ name: 'Dashboard' })
  } else {
    next()
  }
})

export default router
