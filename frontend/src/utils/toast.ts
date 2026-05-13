type ToastType = 'success' | 'error' | 'warning'

function createToast(message: string, type: ToastType) {
  let container = document.querySelector('.toast-container') as HTMLElement
  if (!container) {
    container = document.createElement('div')
    container.className = 'toast-container'
    document.body.appendChild(container)
  }
  const el = document.createElement('div')
  el.className = `toast ${type}`
  el.textContent = message
  container.appendChild(el)
  setTimeout(() => {
    el.style.opacity = '0'
    el.style.transform = 'translateY(-8px)'
    el.style.transition = 'all 0.2s ease-out'
    setTimeout(() => el.remove(), 200)
  }, 2800)
}

export const toast = {
  success: (msg: string) => createToast(msg, 'success'),
  error: (msg: string) => createToast(msg, 'error'),
  warning: (msg: string) => createToast(msg, 'warning'),
}
