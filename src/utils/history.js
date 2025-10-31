/**
 * 历史记录管理工具
 * 用于实现撤销/重做功能
 */

export class HistoryManager {
  constructor(maxSize = 50) {
    this.maxSize = maxSize;
    this.history = []; // 历史记录栈
    this.currentIndex = -1; // 当前位置
  }

  /**
   * 添加新的历史记录
   * @param {any} state 状态快照
   */
  push(state) {
    // 如果当前不在最新位置，删除后面的记录
    if (this.currentIndex < this.history.length - 1) {
      this.history = this.history.slice(0, this.currentIndex + 1);
    }

    // 添加新记录
    this.history.push(this.deepClone(state));
    this.currentIndex++;

    // 限制历史记录大小
    if (this.history.length > this.maxSize) {
      this.history.shift();
      this.currentIndex--;
    }
  }

  /**
   * 撤销
   * @returns {any|null} 上一个状态，如果无法撤销则返回null
   */
  undo() {
    if (!this.canUndo()) {
      return null;
    }

    this.currentIndex--;
    return this.deepClone(this.history[this.currentIndex]);
  }

  /**
   * 重做
   * @returns {any|null} 下一个状态，如果无法重做则返回null
   */
  redo() {
    if (!this.canRedo()) {
      return null;
    }

    this.currentIndex++;
    return this.deepClone(this.history[this.currentIndex]);
  }

  /**
   * 是否可以撤销
   * @returns {boolean}
   */
  canUndo() {
    return this.currentIndex > 0;
  }

  /**
   * 是否可以重做
   * @returns {boolean}
   */
  canRedo() {
    return this.currentIndex < this.history.length - 1;
  }

  /**
   * 获取当前状态
   * @returns {any|null}
   */
  getCurrentState() {
    if (this.currentIndex >= 0 && this.currentIndex < this.history.length) {
      return this.deepClone(this.history[this.currentIndex]);
    }
    return null;
  }

  /**
   * 清空历史记录
   */
  clear() {
    this.history = [];
    this.currentIndex = -1;
  }

  /**
   * 获取历史记录信息
   * @returns {object}
   */
  getInfo() {
    return {
      total: this.history.length,
      current: this.currentIndex,
      canUndo: this.canUndo(),
      canRedo: this.canRedo(),
    };
  }

  /**
   * 深度克隆对象
   * @param {any} obj 要克隆的对象
   * @returns {any}
   */
  deepClone(obj) {
    if (obj === null || typeof obj !== 'object') {
      return obj;
    }

    if (obj instanceof Date) {
      return new Date(obj.getTime());
    }

    if (obj instanceof Array) {
      return obj.map(item => this.deepClone(item));
    }

    if (obj instanceof Object) {
      const clonedObj = {};
      for (const key in obj) {
        if (obj.hasOwnProperty(key)) {
          clonedObj[key] = this.deepClone(obj[key]);
        }
      }
      return clonedObj;
    }

    return obj;
  }
}

/**
 * 创建历史记录管理器
 * @param {number} maxSize 最大历史记录数
 * @returns {HistoryManager}
 */
export function createHistoryManager(maxSize = 50) {
  return new HistoryManager(maxSize);
}
