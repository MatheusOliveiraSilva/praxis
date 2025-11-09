import type { Metadata } from 'next'
import './globals.css'

export const metadata: Metadata = {
  title: 'Praxis AI Agent',
  description: 'Interact with AI agents powered by Praxis',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="pt-BR">
      <body className="antialiased">{children}</body>
    </html>
  )
}

