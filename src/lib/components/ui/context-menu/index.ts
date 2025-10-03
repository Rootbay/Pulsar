import { ContextMenu as ContextMenuPrimitive } from "bits-ui";

import Root from "./context-menu.svelte";
import Trigger from "./context-menu-trigger.svelte";
import Content from "./context-menu-content.svelte";
import Item from "./context-menu-item.svelte";
import Separator from "./context-menu-separator.svelte";
import Label from "./context-menu-label.svelte";
import Shortcut from "./context-menu-shortcut.svelte";

const Sub = ContextMenuPrimitive.Sub;
const SubTrigger = ContextMenuPrimitive.SubTrigger;
const SubContent = ContextMenuPrimitive.SubContent;
const SubContentStatic = ContextMenuPrimitive.SubContentStatic;
const Group = ContextMenuPrimitive.Group;
const CheckboxItem = ContextMenuPrimitive.CheckboxItem;
const CheckboxGroup = ContextMenuPrimitive.CheckboxGroup;
const RadioGroup = ContextMenuPrimitive.RadioGroup;
const RadioItem = ContextMenuPrimitive.RadioItem;
const ContentStatic = ContextMenuPrimitive.ContentStatic;
const Arrow = ContextMenuPrimitive.Arrow;
const Portal = ContextMenuPrimitive.Portal;

export {
	Root,
	Trigger,
	Content,
	Item,
	Separator,
	Label,
	Shortcut,
	Sub,
	SubTrigger,
	SubContent,
	SubContentStatic,
	Group,
	CheckboxItem,
	CheckboxGroup,
	RadioGroup,
	RadioItem,
	ContentStatic,
	Arrow,
	Portal,
	//
	Root as ContextMenu,
	Trigger as ContextMenuTrigger,
	Content as ContextMenuContent,
	Item as ContextMenuItem,
	Separator as ContextMenuSeparator,
	Label as ContextMenuLabel,
	Shortcut as ContextMenuShortcut,
	Sub as ContextMenuSub,
	SubTrigger as ContextMenuSubTrigger,
	SubContent as ContextMenuSubContent,
	SubContentStatic as ContextMenuSubContentStatic,
	Group as ContextMenuGroup,
	CheckboxItem as ContextMenuCheckboxItem,
	CheckboxGroup as ContextMenuCheckboxGroup,
	RadioGroup as ContextMenuRadioGroup,
	RadioItem as ContextMenuRadioItem,
	ContentStatic as ContextMenuContentStatic,
	Arrow as ContextMenuArrow,
	Portal as ContextMenuPortal,
};
