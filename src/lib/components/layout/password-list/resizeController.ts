interface ResizeOptions {
  cssVar?: string;
  minWidth?: number;
  maxWidth?: number;
  selector?: string;
  onResizeEnd?: (width: number) => void;
}

export function createResizeController({
  cssVar = '--passwordList-width',
  minWidth = 200,
  maxWidth = 600,
  selector = '.passwordList',
  onResizeEnd
}: ResizeOptions = {}) {
  let isResizing = false;
  let startX = 0;
  let initialWidth = 0;
  let currentWidth = 0;

  const cleanup = () => {
    if (typeof window === 'undefined') {
      return;
    }

    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
  };

  const handleMouseMove = (event: MouseEvent) => {
    if (!isResizing) return;
    currentWidth = Math.max(minWidth, Math.min(maxWidth, initialWidth + (event.clientX - startX)));
    document.documentElement.style.setProperty(cssVar, `${currentWidth}px`);
  };

  const handleMouseUp = () => {
    if (!isResizing) {
      cleanup();
      return;
    }

    isResizing = false;
    onResizeEnd?.(currentWidth);
    cleanup();
  };

  const start = (event: MouseEvent) => {
    if (typeof window === 'undefined') {
      return;
    }

    const target = document.querySelector<HTMLElement>(selector);
    if (!target) {
      return;
    }

    isResizing = true;
    startX = event.clientX;
    initialWidth = target.offsetWidth;
    currentWidth = initialWidth;

    cleanup();
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  };

  const setWidth = (width: number) => {
    const nextWidth = Math.max(minWidth, Math.min(maxWidth, width));
    document.documentElement.style.setProperty(cssVar, `${nextWidth}px`);
    onResizeEnd?.(nextWidth);
  };

  const destroy = () => {
    cleanup();
  };

  return { start, setWidth, destroy };
}
