export interface Keybind {
  name: string;
  description: string;
  key: string;
}

export const defaultKeybinds: Keybind[] = [
  { name: 'Go Back', description: 'Navigates back to the previous view', key: 'Escape' },
  { name: 'Save Changes', description: 'Saves current settings or data', key: 'Ctrl+S' },
  { name: 'New Item', description: 'Creates a new password item', key: 'Ctrl+N' },
  { name: 'Copy Title', description: 'Copies the title of the selected item', key: 'Ctrl+T' },
  { name: 'Copy Username', description: 'Copies the username of the selected item', key: 'Ctrl+U' },
  { name: 'Copy Password', description: 'Copies the password of the selected item', key: 'Ctrl+P' },
  { name: 'Copy URL', description: 'Copies the URL of the selected item', key: 'Ctrl+L' },
  { name: 'Delete Item', description: 'Deletes the selected password item', key: 'Delete' },
  { name: 'Search', description: 'Focuses on the search bar', key: 'Ctrl+F' },
  { name: 'Select All', description: 'Selects all text in an input field', key: 'Ctrl+A' },
  { name: 'Undo', description: 'Undoes the last action', key: 'Ctrl+Z' },
  { name: 'Redo', description: 'Redoes the last undone action', key: 'Ctrl+Y' },
  { name: 'Cut', description: 'Cuts the selected text', key: 'Ctrl+X' },
  { name: 'Copy', description: 'Copies the selected text', key: 'Ctrl+C' },
  { name: 'Paste', description: 'Pastes the copied text', key: 'Ctrl+V' }
];
