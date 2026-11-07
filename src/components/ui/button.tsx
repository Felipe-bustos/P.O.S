import type { ComponentProps } from 'react'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/utils/cn'

const buttonVariants = cva(
  'inline-flex min-h-13 items-center justify-center gap-2 rounded-xl px-5 py-3 text-lg font-semibold transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 [&_svg]:size-5 [&_svg]:shrink-0',
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground hover:bg-primary/90',
        outline: 'border border-input bg-background text-foreground hover:bg-accent',
      },
    },
    defaultVariants: { variant: 'default' },
  },
)

export function Button({ className, variant, type = 'button', ...props }: ComponentProps<'button'> & VariantProps<typeof buttonVariants>) {
  return <button type={type} data-slot="button" className={cn(buttonVariants({ variant, className }))} {...props} />
}
