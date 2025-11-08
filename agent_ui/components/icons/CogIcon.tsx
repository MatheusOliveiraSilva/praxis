export function CogIcon({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <circle cx="12" cy="12" r="3" />
      <path d="M12 1v6m0 6v6m5.657-13.657l-4.243 4.243m0 4.828l-4.242 4.243m13.657-5.657l-6 0m-6 0l-6 0m13.657 5.657l-4.243-4.243m0-4.828l-4.242-4.243" />
    </svg>
  )
}

