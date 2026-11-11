// Small line icons, drawn here so the app ships no icon library. All share one
// grid, one stroke and round joins.
import type { ReactNode, SVGProps } from 'react';

type IconProps = Omit<SVGProps<SVGSVGElement>, 'children'> & { size?: number };

function make(children: ReactNode) {
  return function Icon({ size = 20, ...props }: IconProps) {
    return (
      <svg
        width={size}
        height={size}
        viewBox="0 0 20 20"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
        strokeLinejoin="round"
        aria-hidden="true"
        focusable="false"
        {...props}
      >
        {children}
      </svg>
    );
  };
}

export const PhoneIcon = make(
  <>
    <rect x="5.5" y="2" width="9" height="16" rx="2" />
    <path d="M9 15.2h2" />
  </>,
);
export const TowerIcon = make(
  <>
    <path d="M10 8v10M7 18h6" />
    <path d="M6.8 10.6a4.5 4.5 0 0 1 0-6.2M13.2 4.4a4.5 4.5 0 0 1 0 6.2" />
    <path d="M4.4 12.6a8 8 0 0 1 0-10.2M15.6 2.4a8 8 0 0 1 0 10.2" />
    <circle cx="10" cy="7.5" r="1" fill="currentColor" stroke="none" />
  </>,
);
export const GlobeIcon = make(
  <>
    <circle cx="10" cy="10" r="7.5" />
    <path d="M2.5 10h15M10 2.5c2.4 2.2 2.4 12.8 0 15M10 2.5c-2.4 2.2-2.4 12.8 0 15" />
  </>,
);
export const SiteIcon = make(
  <>
    <rect x="2.5" y="4" width="15" height="12" rx="2" />
    <path d="M2.5 7.5h15M5 5.8h.01M7 5.8h.01" />
  </>,
);
export const ExchangeIcon = make(
  <>
    <path d="M3.5 17V6.5L10 3l6.5 3.5V17" />
    <path d="M2 17h16M7.5 17v-4h5v4M7.5 9.5h.01M12.5 9.5h.01" />
  </>,
);
export const RouterIcon = make(
  <>
    <rect x="2.5" y="10.5" width="15" height="6" rx="1.5" />
    <path d="M6 13.5h.01M9 13.5h.01M14 10.5V5M11.8 6.2a3 3 0 0 1 4.4 0" />
  </>,
);
export const ServerIcon = make(
  <>
    <rect x="3" y="3" width="14" height="6" rx="1.5" />
    <rect x="3" y="11" width="14" height="6" rx="1.5" />
    <path d="M6 6h.01M6 14h.01M9.5 6H14M9.5 14H14" />
  </>,
);
export const RelayIcon = make(
  <>
    <circle cx="10" cy="10" r="2.6" />
    <path d="M10 2.5v4.9M10 12.6v4.9M2.5 10h4.9M12.6 10h4.9" />
  </>,
);

export const CheckIcon = make(<path d="M4.5 10.5l3.5 3.5 7.5-8" />);
export const CrossIcon = make(<path d="M5.5 5.5l9 9M14.5 5.5l-9 9" />);
export const QuestionIcon = make(
  <>
    <path d="M7.3 7.5a2.8 2.8 0 1 1 4.2 2.4c-.9.6-1.5 1.1-1.5 2.1" />
    <path d="M10 15.2h.01" />
  </>,
);
export const DotIcon = make(<circle cx="10" cy="10" r="2.5" />);

export const PlayIcon = make(<path d="M7 4.5v11l8.5-5.5z" fill="currentColor" />);
export const PauseIcon = make(<path d="M7 5v10M13 5v10" strokeWidth="2.4" />);
export const StepBackIcon = make(<path d="M12.5 5l-6 5 6 5M15.5 5v10" />);
export const StepForwardIcon = make(<path d="M7.5 5l6 5-6 5M4.5 5v10" />);
export const SkipIcon = make(<path d="M4 5l5.5 5L4 15M10.5 5l5.5 5-5.5 5" />);
export const RestartIcon = make(
  <>
    <path d="M4 10a6 6 0 1 0 1.9-4.4" />
    <path d="M3.5 3.5v3h3" />
  </>,
);
export const ArrowRightIcon = make(<path d="M4 10h12M11.5 5.5L16 10l-4.5 4.5" />);
export const ArrowLeftIcon = make(<path d="M16 10H4M8.5 5.5L4 10l4.5 4.5" />);
export const ChevronIcon = make(<path d="M6 8l4 4 4-4" />);
export const CloseIcon = make(<path d="M5 5l10 10M15 5L5 15" />);
export const PlusIcon = make(<path d="M10 4.5v11M4.5 10h11" />);
export const TrashIcon = make(
  <>
    <path d="M4 6h12M8 6V4h4v2M5.5 6l.7 10h7.6l.7-10" />
  </>,
);
export const PencilIcon = make(<path d="M4 16l.8-3.4L13.4 4l2.6 2.6-8.6 8.6zM12 5.5l2.5 2.5" />);
export const GearIcon = make(
  <>
    <circle cx="10" cy="10" r="2.5" />
    <path d="M10 2.5v2M10 15.5v2M2.5 10h2M15.5 10h2M4.7 4.7l1.4 1.4M13.9 13.9l1.4 1.4M4.7 15.3l1.4-1.4M13.9 6.1l1.4-1.4" />
  </>,
);
export const BulbIcon = make(
  <>
    <path d="M7.5 14.5h5M8.5 17h3" />
    <path d="M6 9a4 4 0 1 1 6.5 3.1c-.6.5-.8 1-.9 1.4H8.4c-.1-.4-.3-.9-.9-1.4A4 4 0 0 1 6 9z" />
  </>,
);
export const LockIcon = make(
  <>
    <rect x="4.5" y="9" width="11" height="8" rx="1.5" />
    <path d="M7 9V6.5a3 3 0 0 1 6 0V9" />
  </>,
);

const nodeIcons = {
  phone: PhoneIcon,
  site: SiteIcon,
  server: ServerIcon,
  relay: RelayIcon,
  router: RouterIcon,
  isp: ExchangeIcon,
  mobile: TowerIcon,
  internet: GlobeIcon,
} as const;

/** The picture for a node, from its kind and its optional `icon` hint. */
export function NodeIcon({ kind, hint, size }: { kind: string; hint?: string; size?: number }) {
  const key = (hint && hint in nodeIcons ? hint : kind === 'host' ? 'server' : kind) as keyof typeof nodeIcons;
  const Icon = nodeIcons[key] ?? ServerIcon;
  return <Icon size={size} />;
}
