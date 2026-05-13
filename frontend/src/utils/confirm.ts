export function confirmDialog(message: string): Promise<boolean> {
  return new Promise((resolve) => {
    const overlay = document.createElement('div')
    overlay.className = 'confirm-overlay'
    overlay.innerHTML = `
      <div class="confirm-box">
        <div class="confirm-icon">⚘</div>
        <div class="confirm-msg">${message}</div>
        <div class="confirm-actions">
          <button class="btn" id="cfm-cancel">取消</button>
          <button class="btn btn-primary" id="cfm-ok">确定</button>
        </div>
      </div>
    `
    document.body.appendChild(overlay)

    const cleanup = (result: boolean) => {
      overlay.style.opacity = '0'
      overlay.style.transition = 'opacity 0.15s'
      setTimeout(() => overlay.remove(), 150)
      resolve(result)
    }

    overlay.querySelector('#cfm-cancel')!.addEventListener('click', () => cleanup(false))
    overlay.querySelector('#cfm-ok')!.addEventListener('click', () => cleanup(true))
    overlay.addEventListener('click', (e) => {
      if (e.target === overlay) cleanup(false)
    })
  })
}
