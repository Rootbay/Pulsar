import { iconPaths } from '$lib/icons';
import type { FilterCategory } from '$lib/stores';
import type { PasswordItem } from '../../../../routes/+layout.ts';

export type TagButton = {
  id?: number;
  text: string;
  color: string;
  icon: string;
};

export type TagMeta = {
  icon: string;
  color: string;
};

export type SectionTitle = 'Pinned' | 'Today' | 'Yesterday' | 'Earlier';

export interface ItemSection {
  title: SectionTitle;
  items: PasswordItem[];
}

export const RECENT_FILTER: FilterCategory = 'recent';
export const PIN_TAG_LABEL = 'Pinned';

const DEFAULT_FALLBACK: TagMeta = {
  icon: iconPaths.default,
  color: 'var(--sidebar-border)'
};

const SECTION_ORDER: SectionTitle[] = [PIN_TAG_LABEL, 'Today', 'Yesterday', 'Earlier'];
const RECENT_DAY_WINDOW = 7;
const DAY_IN_MS = 24 * 60 * 60 * 1000;
const PIN_TAG_NAMES = new Set(['pinned', 'pin']);

export function buildTagMap(buttons: TagButton[]): Map<string, TagMeta> {
  return new Map(
    buttons.map((button) => [button.text, { color: button.color, icon: button.icon }])
  );
}

export function partitionItems(list: PasswordItem[]): ItemSection[] {
  const buckets: Record<SectionTitle, PasswordItem[]> = {
    Pinned: [],
    Today: [],
    Yesterday: [],
    Earlier: []
  };

  for (const item of list) {
    if (isPinned(item)) {
      buckets.Pinned.push(item);
    } else if (item.updated_at && isToday(item.updated_at)) {
      buckets.Today.push(item);
    } else if (item.updated_at && isYesterday(item.updated_at)) {
      buckets.Yesterday.push(item);
    } else {
      buckets.Earlier.push(item);
    }
  }

  return SECTION_ORDER.map((title) => ({
    title,
    items: buckets[title]
  }));
}

export function skeletonPlaceholders(count: number): number[] {
  return Array.from({ length: count }, (_, index) => index);
}

export function getTagNames(item: PasswordItem): string[] {
  return item.tags
    ? item.tags
        .split(',')
        .map((tag) => tag.trim())
        .filter(Boolean)
    : [];
}

export function hasPinnedTag(tagNames: string[]): boolean {
  return tagNames.some((tag) => PIN_TAG_NAMES.has(tag.toLowerCase()));
}

export function isPinned(item: PasswordItem): boolean {
  return hasPinnedTag(getTagNames(item));
}

export function toDate(dateStr: string): Date | null {
  const date = new Date(dateStr);
  return Number.isNaN(date.getTime()) ? null : date;
}

export function isSameDay(a: Date, b: Date): boolean {
  return (
    a.getFullYear() === b.getFullYear() &&
    a.getMonth() === b.getMonth() &&
    a.getDate() === b.getDate()
  );
}

export function isToday(dateStr: string): boolean {
  const date = toDate(dateStr);
  if (!date) {
    return false;
  }
  const now = new Date();
  return isSameDay(date, now);
}

export function isYesterday(dateStr: string): boolean {
  const date = toDate(dateStr);
  if (!date) {
    return false;
  }
  const yesterday = new Date();
  yesterday.setDate(yesterday.getDate() - 1);
  return isSameDay(date, yesterday);
}

export function isWithinDays(dateStr: string, windowInDays: number): boolean {
  const date = toDate(dateStr);
  if (!date) {
    return false;
  }
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  if (diff < 0) {
    return false;
  }
  return diff <= windowInDays * DAY_IN_MS;
}

export function isRecent(item: PasswordItem, tagNames: string[]): boolean {
  if (hasPinnedTag(tagNames)) {
    return true;
  }

  return item.updated_at ? isWithinDays(item.updated_at, RECENT_DAY_WINDOW) : false;
}

export function getFallback(item: PasswordItem, tagMap: Map<string, TagMeta>): TagMeta {
  const tagNames = getTagNames(item);
  if (tagNames.length > 0) {
    const firstTagMeta = tagMap.get(tagNames[0]);
    if (firstTagMeta) {
      return firstTagMeta;
    }
  }
  return DEFAULT_FALLBACK;
}
