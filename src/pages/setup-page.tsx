import { useState } from 'react'
import { isTauri } from '@tauri-apps/api/core'
import { CheckCircle2, Database, LoaderCircle, Store } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { checkLocalStorage } from '@/services/health-service'

export function SetupPage() {
  const [status, setStatus] = useState<'idle' | 'checking' | 'ready' | 'error'>('idle')
  const [error, setError] = useState('')
  const desktop = isTauri()

  async function handleStorageCheck() {
    setStatus('checking')
    setError('')
    try {
      await checkLocalStorage()
      setStatus('ready')
      toast.success('El almacenamiento local funciona correctamente')
    } catch (cause) {
      setStatus('error')
      setError(cause instanceof Error ? cause.message : 'No se pudo comprobar el almacenamiento. Intenta nuevamente.')
    }
  }

  return (
    <main className="min-h-screen bg-background text-foreground">
      <header className="border-b border-border bg-card px-6 py-5">
        <div className="mx-auto flex max-w-4xl items-center gap-3">
          <Store className="size-9 text-primary" aria-hidden="true" />
          <span className="text-2xl font-bold">Almacén Local</span>
        </div>
      </header>
      <div className="mx-auto max-w-4xl px-6 py-12 sm:py-20">
        <p className="mb-3 text-lg font-semibold text-primary">Preparación inicial</p>
        <h1 className="text-3xl font-bold tracking-tight sm:text-4xl">Todo comienza en tu almacén</h1>
        <p className="mt-4 max-w-2xl text-xl leading-relaxed text-muted-foreground">
          Esta es la base de tu aplicación. Antes de agregar productos y registrar ventas, comprueba que puedes guardar información en este computador.
        </p>
        <section className="mt-10 rounded-2xl border border-border bg-card p-6 sm:p-8" aria-labelledby="storage-title">
          <Database className="mb-4 size-8 text-primary" aria-hidden="true" />
          <h2 id="storage-title" className="text-2xl font-bold">Almacenamiento local</h2>
          <p className="mt-3 text-lg leading-relaxed text-muted-foreground">
            La comprobación guarda y lee un dato de prueba. No necesita conexión a internet.
          </p>
          {!desktop && <p className="mt-5 rounded-xl bg-muted p-4 text-lg">Estás viendo la vista previa en el navegador. Abre la aplicación de escritorio para comprobar el almacenamiento.</p>}
          <Button className="mt-6 w-full sm:w-auto" onClick={handleStorageCheck} disabled={!desktop || status === 'checking'}>
            {status === 'checking' ? <LoaderCircle className="animate-spin motion-reduce:animate-none" aria-hidden="true" /> : <Database aria-hidden="true" />}
            {status === 'checking' ? 'Comprobando…' : status === 'error' ? 'Volver a comprobar' : 'Comprobar almacenamiento'}
          </Button>
          <div role="status" aria-live="polite" className="mt-4 text-lg">
            {status === 'ready' && <p className="flex items-center gap-2 font-semibold text-green-800"><CheckCircle2 aria-hidden="true" />El dato se guardó y se leyó correctamente.</p>}
          </div>
          {status === 'error' && <p role="alert" className="mt-4 break-words text-lg text-destructive">{error}</p>}
        </section>
        <p className="mt-6 text-lg text-muted-foreground">Próximo paso: catálogo de productos y pantalla de venta.</p>
      </div>
    </main>
  )
}
