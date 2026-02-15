import { cn } from "@/lib/utils"

interface LogoProps {
  className?: string
  size?: number
}

export function Logo({ className, size = 28 }: LogoProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 64 64"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={cn("shrink-0", className)}
    >
      {/* Background rounded square */}
      <rect
        x="2"
        y="2"
        width="60"
        height="60"
        rx="14"
        fill="url(#logo-gradient)"
      />

      {/* Subtle inner glow */}
      <rect
        x="2"
        y="2"
        width="60"
        height="60"
        rx="14"
        fill="url(#logo-shine)"
        opacity="0.5"
      />

      {/* Code bracket left  < */}
      <path
        d="M24 21L13 32L24 43"
        stroke="white"
        strokeWidth="4"
        strokeLinecap="round"
        strokeLinejoin="round"
        opacity="0.9"
      />

      {/* Code bracket right  > */}
      <path
        d="M40 21L51 32L40 43"
        stroke="white"
        strokeWidth="4"
        strokeLinecap="round"
        strokeLinejoin="round"
        opacity="0.9"
      />

      {/* Server/stack lines in center */}
      <rect x="28" y="25" width="8" height="2.5" rx="1.25" fill="white" opacity="0.95" />
      <rect x="28" y="30.75" width="8" height="2.5" rx="1.25" fill="white" opacity="0.75" />
      <rect x="28" y="36.5" width="8" height="2.5" rx="1.25" fill="white" opacity="0.55" />

      <defs>
        <linearGradient id="logo-gradient" x1="2" y1="2" x2="62" y2="62" gradientUnits="userSpaceOnUse">
          <stop stopColor="#1e293b" />
          <stop offset="1" stopColor="#0f172a" />
        </linearGradient>
        <linearGradient id="logo-shine" x1="32" y1="2" x2="32" y2="62" gradientUnits="userSpaceOnUse">
          <stop stopColor="white" stopOpacity="0.12" />
          <stop offset="1" stopColor="white" stopOpacity="0" />
        </linearGradient>
      </defs>
    </svg>
  )
}
