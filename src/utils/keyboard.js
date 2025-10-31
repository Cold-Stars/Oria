/**
 * 键盘快捷键管理工具
 */

export class KeyboardManager {
    constructor() {
        this.shortcuts = new Map();
        this.isListening = false;
        this.handleKeyDown = this.handleKeyDown.bind(this);
    }

    /**
     * 注册快捷键
     * @param {string} key 按键组合，如 'ctrl+s', 'alt+n'
     * @param {function} callback 回调函数
     * @param {string} description 描述
     */
    register(key, callback, description = '') {
        const normalizedKey = this.normalizeKey(key);
        this.shortcuts.set(normalizedKey, { callback, description });
    }

    /**
     * 注销快捷键
     * @param {string} key 按键组合
     */
    unregister(key) {
        const normalizedKey = this.normalizeKey(key);
        this.shortcuts.delete(normalizedKey);
    }

    /**
     * 开始监听键盘事件
     */
    startListening() {
        if (!this.isListening) {
            document.addEventListener('keydown', this.handleKeyDown);
            this.isListening = true;
        }
    }

    /**
     * 停止监听键盘事件
     */
    stopListening() {
        if (this.isListening) {
            document.removeEventListener('keydown', this.handleKeyDown);
            this.isListening = false;
        }
    }

    /**
     * 处理键盘事件
     * @param {KeyboardEvent} event 键盘事件
     */
    handleKeyDown(event) {
        // 忽略在输入框中的按键
        if (this.isInputElement(event.target)) {
            return;
        }

        const key = this.getKeyFromEvent(event);
        const shortcut = this.shortcuts.get(key);

        if (shortcut) {
            event.preventDefault();
            shortcut.callback(event);
        }
    }

    /**
     * 标准化按键字符串
     * @param {string} key 按键组合
     * @returns {string}
     */
    normalizeKey(key) {
        return key.toLowerCase()
            .replace(/\s/g, '')
            .split('+')
            .sort()
            .join('+');
    }

    /**
     * 从事件对象获取按键组合字符串
     * @param {KeyboardEvent} event 键盘事件
     * @returns {string}
     */
    getKeyFromEvent(event) {
        const parts = [];

        if (event.ctrlKey) parts.push('ctrl');
        if (event.altKey) parts.push('alt');
        if (event.shiftKey) parts.push('shift');
        if (event.metaKey) parts.push('meta');

        // 添加主按键
        const key = event.key.toLowerCase();
        if (key !== 'control' && key !== 'alt' && key !== 'shift' && key !== 'meta') {
            parts.push(key);
        }

        return parts.sort().join('+');
    }

    /**
     * 检查目标元素是否为输入元素
     * @param {Element} element DOM元素
     * @returns {boolean}
     */
    isInputElement(element) {
        const tagName = element.tagName.toLowerCase();
        const inputTypes = ['input', 'textarea', 'select'];

        if (inputTypes.includes(tagName)) {
            return true;
        }

        // 检查contenteditable
        if (element.contentEditable === 'true') {
            return true;
        }

        return false;
    }

    /**
     * 获取所有已注册的快捷键
     * @returns {Array}
     */
    getShortcuts() {
        return Array.from(this.shortcuts.entries()).map(([key, data]) => ({
            key,
            description: data.description
        }));
    }

    /**
     * 销毁管理器
     */
    destroy() {
        this.stopListening();
        this.shortcuts.clear();
    }
}

// 默认快捷键配置
export const DEFAULT_SHORTCUTS = {
    // 文件操作
    'ctrl+o': { action: 'openFolder', description: '打开文件夹' },
    'ctrl+s': { action: 'save', description: '保存标注' },

    // 工具切换
    's': { action: 'selectTool', description: '选择工具' },
    'r': { action: 'rectangleTool', description: '矩形标注工具' },
    'o': { action: 'rotatedRectangleTool', description: '旋转矩形工具' },
    'n': { action: 'createAnnotation', description: '创建标注（使用最近的工具）' },

    // 导航
    'arrowleft': { action: 'prevImage', description: '上一张图片' },
    'arrowright': { action: 'nextImage', description: '下一张图片' },
    'home': { action: 'firstImage', description: '第一张图片' },
    'end': { action: 'lastImage', description: '最后一张图片' },

    // 标注操作
    'delete': { action: 'deleteAnnotation', description: '删除选中标注' },
    'ctrl+a': { action: 'selectAll', description: '选择所有标注' },
    'ctrl+d': { action: 'duplicate', description: '复制标注' },
    'escape': { action: 'clearSelection', description: '清除选择' },

    // 视图操作
    'ctrl+0': { action: 'resetZoom', description: '重置缩放' },
    'ctrl+1': { action: 'fitToWindow', description: '适应窗口' },
    'ctrl+=': { action: 'zoomIn', description: '放大' },
    'ctrl+-': { action: 'zoomOut', description: '缩小' },
    'h': { action: 'toggleAnnotations', description: '显示/隐藏标注' },

    // 其他
    'f1': { action: 'showHelp', description: '显示快捷键帮助' },
    'ctrl+z': { action: 'undo', description: '撤销' },
    'ctrl+y': { action: 'redo', description: '重做' },
};

/**
 * 创建并配置键盘管理器
 * @param {object} callbacks 回调函数映射
 * @returns {KeyboardManager}
 */
export function createKeyboardManager(callbacks = {}) {
    const manager = new KeyboardManager();

    // 注册默认快捷键
    Object.entries(DEFAULT_SHORTCUTS).forEach(([key, config]) => {
        const callback = callbacks[config.action];
        if (callback && typeof callback === 'function') {
            manager.register(key, callback, config.description);
        }
    });

    return manager;
}

/**
 * 格式化快捷键显示文本
 * @param {string} key 按键组合
 * @returns {string}
 */
export function formatShortcutKey(key) {
    return key
        .split('+')
        .map(k => {
            switch (k.toLowerCase()) {
                case 'ctrl': return 'Ctrl';
                case 'alt': return 'Alt';
                case 'shift': return 'Shift';
                case 'meta': return 'Cmd';
                case 'arrowleft': return '←';
                case 'arrowright': return '→';
                case 'arrowup': return '↑';
                case 'arrowdown': return '↓';
                case 'escape': return 'Esc';
                case 'delete': return 'Del';
                case 'backspace': return 'Backspace';
                default: return k.toUpperCase();
            }
        })
        .join(' + ');
} 