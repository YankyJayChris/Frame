import * as vscode from 'vscode';

interface IconEntry {
  name: string;
  category: string;
  sfSymbol?: string;
  materialIcon?: string;
}

const ICON_CATEGORIES: IconEntry[] = [
  { name: 'plus', category: 'Actions', sfSymbol: 'plus', materialIcon: 'add' },
  { name: 'minus', category: 'Actions', sfSymbol: 'minus', materialIcon: 'remove' },
  { name: 'checkmark', category: 'Actions', sfSymbol: 'checkmark', materialIcon: 'check' },
  { name: 'xmark', category: 'Actions', sfSymbol: 'xmark', materialIcon: 'close' },
  { name: 'trash', category: 'Actions', sfSymbol: 'trash', materialIcon: 'delete' },
  { name: 'pencil', category: 'Actions', sfSymbol: 'pencil', materialIcon: 'edit' },
  { name: 'plus.circle', category: 'Actions', sfSymbol: 'plus.circle', materialIcon: 'add_circle' },
  { name: 'checkmark.circle', category: 'Actions', sfSymbol: 'checkmark.circle', materialIcon: 'check_circle' },
  { name: 'xmark.circle', category: 'Actions', sfSymbol: 'xmark.circle', materialIcon: 'cancel' },
  { name: 'ellipsis', category: 'Actions', sfSymbol: 'ellipsis', materialIcon: 'more_horiz' },
  { name: 'square.and.arrow.up', category: 'Actions', sfSymbol: 'square.and.arrow.up', materialIcon: 'share' },
  { name: 'link', category: 'Actions', sfSymbol: 'link', materialIcon: 'link' },
  { name: 'magnifyingglass', category: 'Search', sfSymbol: 'magnifyingglass', materialIcon: 'search' },
  { name: 'search', category: 'Search', sfSymbol: 'search', materialIcon: 'manage_search' },
  { name: 'slider.horizontal.3', category: 'Search', sfSymbol: 'slider.horizontal.3', materialIcon: 'tune' },
  { name: 'line.3.horizontal', category: 'Navigation', sfSymbol: 'line.3.horizontal', materialIcon: 'menu' },
  { name: 'list.bullet', category: 'Navigation', sfSymbol: 'list.bullet', materialIcon: 'list' },
  { name: 'house', category: 'Navigation', sfSymbol: 'house', materialIcon: 'home' },
  { name: 'house.fill', category: 'Navigation', sfSymbol: 'house.fill', materialIcon: 'home_filled' },
  { name: 'square.grid.2x2', category: 'Navigation', sfSymbol: 'square.grid.2x2', materialIcon: 'apps' },
  { name: 'person', category: 'People', sfSymbol: 'person', materialIcon: 'person' },
  { name: 'person.2', category: 'People', sfSymbol: 'person.2', materialIcon: 'group' },
  { name: 'person.circle', category: 'People', sfSymbol: 'person.circle', materialIcon: 'account_circle' },
  { name: 'star', category: 'Rating', sfSymbol: 'star', materialIcon: 'star' },
  { name: 'star.fill', category: 'Rating', sfSymbol: 'star.fill', materialIcon: 'star_filled' },
  { name: 'heart', category: 'Rating', sfSymbol: 'heart', materialIcon: 'favorite' },
  { name: 'heart.fill', category: 'Rating', sfSymbol: 'heart.fill', materialIcon: 'favorite_filled' },
  { name: 'gear', category: 'Settings', sfSymbol: 'gear', materialIcon: 'settings' },
  { name: 'gearshape', category: 'Settings', sfSymbol: 'gearshape', materialIcon: 'build' },
  { name: 'bell', category: 'Notifications', sfSymbol: 'bell', materialIcon: 'notifications' },
  { name: 'bell.fill', category: 'Notifications', sfSymbol: 'bell.fill', materialIcon: 'notifications_filled' },
  { name: 'bell.badge', category: 'Notifications', sfSymbol: 'bell.badge', materialIcon: 'notifications_active' },
  { name: 'envelope', category: 'Communication', sfSymbol: 'envelope', materialIcon: 'mail' },
  { name: 'envelope.fill', category: 'Communication', sfSymbol: 'envelope.fill', materialIcon: 'mail_filled' },
  { name: 'phone', category: 'Communication', sfSymbol: 'phone', materialIcon: 'phone' },
  { name: 'phone.fill', category: 'Communication', sfSymbol: 'phone.fill', materialIcon: 'phone_filled' },
  { name: 'message', category: 'Communication', sfSymbol: 'message', materialIcon: 'chat' },
  { name: 'bubble.left', category: 'Communication', sfSymbol: 'bubble.left', materialIcon: 'chat_bubble' },
  { name: 'cloud', category: 'Status', sfSymbol: 'cloud', materialIcon: 'cloud' },
  { name: 'cloud.fill', category: 'Status', sfSymbol: 'cloud.fill', materialIcon: 'cloud_filled' },
  { name: 'sun.max', category: 'Weather', sfSymbol: 'sun.max', materialIcon: 'sunny' },
  { name: 'moon', category: 'Weather', sfSymbol: 'moon', materialIcon: 'dark_mode' },
  { name: 'play', category: 'Media', sfSymbol: 'play', materialIcon: 'play_arrow' },
  { name: 'pause', category: 'Media', sfSymbol: 'pause', materialIcon: 'pause' },
  { name: 'stop', category: 'Media', sfSymbol: 'stop', materialIcon: 'stop' },
  { name: 'forward', category: 'Media', sfSymbol: 'forward', materialIcon: 'fast_forward' },
  { name: 'backward', category: 'Media', sfSymbol: 'backward', materialIcon: 'fast_rewind' },
  { name: 'camera', category: 'Media', sfSymbol: 'camera', materialIcon: 'camera' },
  { name: 'photo', category: 'Media', sfSymbol: 'photo', materialIcon: 'photo_library' },
  { name: 'doc', category: 'Documents', sfSymbol: 'doc', materialIcon: 'description' },
  { name: 'folder', category: 'Documents', sfSymbol: 'folder', materialIcon: 'folder' },
  { name: 'folder.fill', category: 'Documents', sfSymbol: 'folder.fill', materialIcon: 'folder_filled' },
  { name: 'map', category: 'Maps', sfSymbol: 'map', materialIcon: 'map' },
  { name: 'location', category: 'Maps', sfSymbol: 'location', materialIcon: 'location_on' },
  { name: 'location.fill', category: 'Maps', sfSymbol: 'location.fill', materialIcon: 'my_location' },
  { name: 'lock', category: 'Security', sfSymbol: 'lock', materialIcon: 'lock' },
  { name: 'lock.fill', category: 'Security', sfSymbol: 'lock.fill', materialIcon: 'lock_filled' },
  { name: 'lock.open', category: 'Security', sfSymbol: 'lock.open', materialIcon: 'lock_open' },
  { name: 'eye', category: 'Actions', sfSymbol: 'eye', materialIcon: 'visibility' },
  { name: 'eye.slash', category: 'Actions', sfSymbol: 'eye.slash', materialIcon: 'visibility_off' },
  { name: 'tag', category: 'Actions', sfSymbol: 'tag', materialIcon: 'sell' },
  { name: 'bookmark', category: 'Actions', sfSymbol: 'bookmark', materialIcon: 'bookmark' },
  { name: 'flag', category: 'Actions', sfSymbol: 'flag', materialIcon: 'flag' },
  { name: 'clock', category: 'Time', sfSymbol: 'clock', materialIcon: 'schedule' },
  { name: 'calendar', category: 'Time', sfSymbol: 'calendar', materialIcon: 'calendar_today' },
  { name: 'gift', category: 'Objects', sfSymbol: 'gift', materialIcon: 'card_giftcard' },
  { name: 'cart', category: 'Commerce', sfSymbol: 'cart', materialIcon: 'shopping_cart' },
  { name: 'bag', category: 'Commerce', sfSymbol: 'bag', materialIcon: 'shopping_bag' },
  { name: 'creditcard', category: 'Commerce', sfSymbol: 'creditcard', materialIcon: 'credit_card' },
  { name: 'wifi', category: 'Status', sfSymbol: 'wifi', materialIcon: 'wifi' },
  { name: 'battery.100', category: 'Status', sfSymbol: 'battery.100', materialIcon: 'battery_full' },
  { name: 'info', category: 'Info', sfSymbol: 'info', materialIcon: 'info' },
  { name: 'info.circle', category: 'Info', sfSymbol: 'info.circle', materialIcon: 'info_filled' },
  { name: 'questionmark', category: 'Info', sfSymbol: 'questionmark', materialIcon: 'help' },
  { name: 'questionmark.circle', category: 'Info', sfSymbol: 'questionmark.circle', materialIcon: 'help_filled' },
  { name: 'exclamationmark', category: 'Info', sfSymbol: 'exclamationmark', materialIcon: 'warning' },
  { name: 'exclamationmark.triangle', category: 'Info', sfSymbol: 'exclamationmark.triangle', materialIcon: 'error' },
  { name: 'arrow.up', category: 'Arrows', sfSymbol: 'arrow.up', materialIcon: 'arrow_upward' },
  { name: 'arrow.down', category: 'Arrows', sfSymbol: 'arrow.down', materialIcon: 'arrow_downward' },
  { name: 'arrow.left', category: 'Arrows', sfSymbol: 'arrow.left', materialIcon: 'arrow_back' },
  { name: 'arrow.right', category: 'Arrows', sfSymbol: 'arrow.right', materialIcon: 'arrow_forward' },
  { name: 'chevron.up', category: 'Arrows', sfSymbol: 'chevron.up', materialIcon: 'expand_less' },
  { name: 'chevron.down', category: 'Arrows', sfSymbol: 'chevron.down', materialIcon: 'expand_more' },
  { name: 'chevron.left', category: 'Arrows', sfSymbol: 'chevron.left', materialIcon: 'chevron_left' },
  { name: 'chevron.right', category: 'Arrows', sfSymbol: 'chevron.right', materialIcon: 'chevron_right' },
  { name: 'mic', category: 'Media', sfSymbol: 'mic', materialIcon: 'mic' },
  { name: 'video', category: 'Media', sfSymbol: 'video', materialIcon: 'videocam' },
  { name: 'qrcode', category: 'Actions', sfSymbol: 'qrcode', materialIcon: 'qr_code' },
];

export function registerIconPicker(context: vscode.ExtensionContext): vscode.Disposable[] {
  const disposables: vscode.Disposable[] = [];

  disposables.push(
    vscode.commands.registerCommand('frame.showIcons', async () => {
      const categories = [...new Set(ICON_CATEGORIES.map(i => i.category))];

      const quickPick = vscode.window.createQuickPick();
      quickPick.title = 'Frame Icon Browser';
      quickPick.placeholder = 'Search icons...';
      quickPick.matchOnDescription = true;
      quickPick.matchOnDetail = true;
      quickPick.items = [];

      const allItems: (vscode.QuickPickItem & { icon: IconEntry })[] = [];

      for (const category of categories) {
        const categoryIcons = ICON_CATEGORIES.filter(i => i.category === category);
        allItems.push({
          label: `$(symbol-color) ${category}`,
          kind: vscode.QuickPickItemKind.Separator,
          icon: null as unknown as IconEntry,
        });
        for (const icon of categoryIcons) {
          allItems.push({
            label: icon.name,
            description: `SF: ${icon.sfSymbol}`,
            detail: `Material: ${icon.materialIcon}`,
            icon: icon,
          });
        }
      }

      quickPick.items = allItems;

      quickPick.onDidAccept(() => {
        const selection = quickPick.activeItems[0];
        if (!selection || selection.kind === vscode.QuickPickItemKind.Separator) return;

        const iconData = (selection as any).icon as IconEntry;
        if (iconData) {
          const editor = vscode.window.activeTextEditor;
          if (editor) {
            editor.edit(edit => {
              edit.insert(editor.selection.active, iconData.name);
            });
          }
        }
        quickPick.dispose();
      });

      quickPick.onDidChangeValue(() => {
        const search = quickPick.value.toLowerCase();
        if (!search) {
          quickPick.items = allItems;
          return;
        }

        const filtered: (vscode.QuickPickItem & { icon: IconEntry })[] = [];
        for (const item of allItems) {
          if (item.kind === vscode.QuickPickItemKind.Separator) continue;
          if (
            item.label.toLowerCase().includes(search) ||
            item.description?.toLowerCase().includes(search) ||
            item.detail?.toLowerCase().includes(search)
          ) {
            filtered.push(item);
          }
        }
        quickPick.items = filtered;
      });

      quickPick.show();
    })
  );

  return disposables;
}
