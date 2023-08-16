import { Stack, Typography } from '@mui/material'
import Link from 'next/link'
import { useRouter } from 'next/router'
import { cloneElement } from 'react'
import { twMerge } from 'tailwind-merge'

interface Props extends React.AnchorHTMLAttributes<HTMLAnchorElement> {
  href: string
  icon: JSX.Element
}

export default function NavbarLink(props: Props) {
  const router = useRouter()

  const currentPath = router.pathname
  const selected = currentPath.startsWith(props.href)

  return (
    <Link
      href={props.href}
      className={twMerge(
        'py-2 px-2',
        'text-gray-400',
        'transition-colors',
        'active:text-gray-200',
        'hover:text-white',
        'hover:bg-zinc-800',
        'rounded-lg',
        selected && 'text-white',
        selected && 'bg-zinc-800',
        props.className
      )}
    >
      <Stack direction="row" px={4} alignItems="center" justifyContent="center">
        <Typography variant="body1" className="hidden md:block">{props.children}</Typography>
        {cloneElement(props.icon, {
          className: twMerge(props.icon.props.className, 'block', 'fill-current', 'h-4', 'w-4', 'md:hidden'),
        })}
      </Stack>
    </Link>
  )
}
