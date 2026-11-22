<script setup lang="ts">
interface Props {
  show: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  variant?: 'danger' | 'primary';
}

interface Emits {
  confirm: [];
  cancel: [];
  'update:show': [boolean];
}

const props = withDefaults(defineProps<Props>(), {
  confirmText: 'Confirm',
  cancelText: 'Cancel',
  variant: 'primary',
});

const emit = defineEmits<Emits>();

function handleConfirm() {
  emit('confirm');
  emit('update:show', false);
}

function handleCancel() {
  emit('cancel');
  emit('update:show', false);
}
</script>

<template>
  <Transition name="fade">
    <div class="modal-overlay" v-if="show" @click="handleCancel">
      <div class="modal" @click.stop role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <div class="modal-header">
          <h3 id="modal-title">{{ title }}</h3>
          <button class="modal-close" @click="handleCancel" aria-label="Close">
            <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>
        <div class="modal-body">
          <p>{{ message }}</p>
        </div>
        <div class="modal-footer">
          <button class="btn btn-secondary" @click="handleCancel">{{ cancelText }}</button>
          <button 
            class="btn" 
            :class="variant === 'danger' ? 'btn-danger' : 'btn-primary'" 
            @click="handleConfirm"
          >
            {{ confirmText }}
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 150ms ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.modal {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: 0 20px 40px var(--shadow);
  min-width: 360px;
  max-width: 480px;
  width: 100%;
  overflow: hidden;
  transform: translateY(0);
}

.fade-enter-from .modal,
.fade-leave-to .modal {
  transform: translateY(-20px);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.modal-header h3 {
  font-size: 15px;
  font-weight: 600;
  color: var(--fg-primary);
}

.modal-close {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--fg-muted);
}

.modal-close:hover {
  background: var(--bg-hover);
  color: var(--fg-primary);
}

.modal-body {
  padding: 20px;
}

.modal-body p {
  font-size: 13px;
  color: var(--fg-secondary);
  line-height: 1.6;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
  background: var(--bg-tertiary);
}
</style>