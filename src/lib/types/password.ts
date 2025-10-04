export interface PasswordItem {
	id: number;
	title: string;
	description: string | null;
	img: string | null;
	tags: string | null;
	username: string | null;
	url: string | null;
	notes: string | null;
	password: string;
	totp_secret?: string | null;
	created_at: string;
	updated_at: string;
	color: string | null;
	custom_fields: { name: string; value: string; field_type: string }[];
	field_order?: string[] | null;
}
