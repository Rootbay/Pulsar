import type { PasswordItem } from '$lib/types/password';
import type { DisplayField } from '$lib/types/password-fields';

export function buildDisplayFields(
	item: PasswordItem | null,
	icons: Record<string, string>
): DisplayField[] {
	if (!item) {
		return [];
	}

	const staticFields: DisplayField[] = [
		{
			id: 'username',
			name: 'Username',
			value: item.username ?? null,
			type: 'text',
			icon: icons.user
		},
		{
			id: 'password',
			name: 'Password',
			value: item.password ?? null,
			type: 'password',
			icon: icons.key
		},
		{
			id: 'url',
			name: 'URL',
			value: item.url ?? null,
			type: 'text',
			icon: icons.link
		},
		{
			id: 'notes',
			name: 'Notes',
			value: item.notes ?? null,
			type: 'multiline',
			icon: icons.notes
		}
	];

	const customFields: DisplayField[] = (item.custom_fields ?? []).map((field) => ({
		id: field.name,
		name: field.name,
		value: field.value ?? null,
		type: field.field_type,
		icon: icons.plus
	}));

	let allFields = [...staticFields, ...customFields];

	if (item.field_order?.length) {
		const ordered: DisplayField[] = [];
		const map = new Map(allFields.map((field) => [field.id, field]));

		for (const fieldId of item.field_order) {
			const field = map.get(fieldId);
			if (field) {
				ordered.push(field);
				map.delete(fieldId);
			}
		}

		ordered.push(...map.values());
		allFields = ordered;
	}

	return allFields;
}
