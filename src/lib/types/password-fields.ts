export interface BaseDisplayField {
	id: string;
	name: string;
	value: string | null;
	type: string;
	icon: string;
}

export interface TotpFieldMeta {
	secret: string;
	digits: number;
	period: number;
}

export interface TotpDisplayField extends Omit<BaseDisplayField, 'type' | 'value'> {
	type: 'totp';
	value: string;
	meta: TotpFieldMeta;
}

export type DisplayField = BaseDisplayField | TotpDisplayField;

export function isTotpDisplayField(field: DisplayField): field is TotpDisplayField {
	return field.type === 'totp';
}
