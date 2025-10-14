/**
 * 标注相关的工具函数
 */

// 标注类型
export const AnnotationType = {
    RECTANGLE: 'rectangle',
    ROTATED_RECTANGLE: 'rotated-rectangle'
};

// 支持的图片格式
export const SUPPORTED_IMAGE_FORMATS = [
    'jpg', 'jpeg', 'png', 'bmp', 'gif', 'tiff', 'webp'
];

/**
 * 检查文件是否为支持的图片格式
 * @param {string} filename 文件名
 * @returns {boolean}
 */
export function isImageFile(filename) {
    const extension = filename.split('.').pop()?.toLowerCase();
    return SUPPORTED_IMAGE_FORMATS.includes(extension);
}

/**
 * 创建新的标注对象
 * @param {string} type 标注类型
 * @param {number} x X坐标
 * @param {number} y Y坐标
 * @param {number} width 宽度
 * @param {number} height 高度
 * @param {string} label 标签
 * @param {number} rotation 旋转角度（仅旋转矩形）
 * @returns {object}
 */
export function createAnnotation(type, x, y, width, height, label = '', rotation = 0) {
    const annotation = {
        id: Date.now() + Math.random(),
        type,
        x: Math.round(x),
        y: Math.round(y),
        width: Math.round(width),
        height: Math.round(height),
        label,
        created: new Date().toISOString()
    };

    if (type === AnnotationType.ROTATED_RECTANGLE) {
        annotation.rotation = rotation;
    }

    return annotation;
}

/**
 * 计算标注的面积
 * @param {object} annotation 标注对象
 * @returns {number}
 */
export function calculateArea(annotation) {
    return annotation.width * annotation.height;
}

/**
 * 计算标注的中心点
 * @param {object} annotation 标注对象
 * @returns {object}
 */
export function calculateCenter(annotation) {
    return {
        x: annotation.x + annotation.width / 2,
        y: annotation.y + annotation.height / 2
    };
}

/**
 * 检查点是否在矩形内
 * @param {number} x 点的X坐标
 * @param {number} y 点的Y坐标
 * @param {object} annotation 标注对象
 * @returns {boolean}
 */
export function isPointInRectangle(x, y, annotation) {
    if (annotation.type === AnnotationType.RECTANGLE) {
        return x >= annotation.x &&
            x <= annotation.x + annotation.width &&
            y >= annotation.y &&
            y <= annotation.y + annotation.height;
    }

    if (annotation.type === AnnotationType.ROTATED_RECTANGLE) {
        // 旋转矩形的点击检测需要逆向旋转点坐标
        const center = calculateCenter(annotation);
        const cos = Math.cos(-annotation.rotation);
        const sin = Math.sin(-annotation.rotation);

        // 将点相对于中心点进行逆向旋转
        const dx = x - center.x;
        const dy = y - center.y;
        const rotatedX = dx * cos - dy * sin + center.x;
        const rotatedY = dx * sin + dy * cos + center.y;

        // 检查旋转后的点是否在原始矩形内
        return rotatedX >= annotation.x &&
            rotatedX <= annotation.x + annotation.width &&
            rotatedY >= annotation.y &&
            rotatedY <= annotation.y + annotation.height;
    }

    return false;
}

/**
 * 获取标注的边界框
 * @param {object} annotation 标注对象
 * @returns {object}
 */
export function getBoundingBox(annotation) {
    if (annotation.type === AnnotationType.RECTANGLE) {
        return {
            left: annotation.x,
            top: annotation.y,
            right: annotation.x + annotation.width,
            bottom: annotation.y + annotation.height
        };
    }

    if (annotation.type === AnnotationType.ROTATED_RECTANGLE) {
        // 计算旋转矩形的边界框
        const center = calculateCenter(annotation);
        const cos = Math.cos(annotation.rotation);
        const sin = Math.sin(annotation.rotation);

        const halfWidth = annotation.width / 2;
        const halfHeight = annotation.height / 2;

        // 四个角点
        const corners = [
            { x: -halfWidth, y: -halfHeight },
            { x: halfWidth, y: -halfHeight },
            { x: halfWidth, y: halfHeight },
            { x: -halfWidth, y: halfHeight }
        ];

        // 旋转并转换到世界坐标
        const rotatedCorners = corners.map(corner => ({
            x: corner.x * cos - corner.y * sin + center.x,
            y: corner.x * sin + corner.y * cos + center.y
        }));

        // 计算边界框
        const xs = rotatedCorners.map(c => c.x);
        const ys = rotatedCorners.map(c => c.y);

        return {
            left: Math.min(...xs),
            top: Math.min(...ys),
            right: Math.max(...xs),
            bottom: Math.max(...ys)
        };
    }

    return null;
}

/**
 * 标注数据验证
 * @param {object} annotation 标注对象
 * @returns {boolean}
 */
export function validateAnnotation(annotation) {
    if (!annotation || typeof annotation !== 'object') {
        return false;
    }

    const requiredFields = ['id', 'type', 'x', 'y', 'width', 'height'];

    for (const field of requiredFields) {
        if (!(field in annotation)) {
            return false;
        }
    }

    // 检查数值字段
    const numericFields = ['x', 'y', 'width', 'height'];
    for (const field of numericFields) {
        if (typeof annotation[field] !== 'number' || isNaN(annotation[field])) {
            return false;
        }
    }

    // 检查宽度和高度必须为正数
    if (annotation.width <= 0 || annotation.height <= 0) {
        return false;
    }

    // 检查类型
    if (!Object.values(AnnotationType).includes(annotation.type)) {
        return false;
    }

    // 旋转矩形需要rotation字段
    if (annotation.type === AnnotationType.ROTATED_RECTANGLE) {
        if (typeof annotation.rotation !== 'number' || isNaN(annotation.rotation)) {
            return false;
        }
    }

    return true;
}

/**
 * 将标注数据转换为YOLO格式
 * @param {object} annotation 标注对象
 * @param {number} imageWidth 图片宽度
 * @param {number} imageHeight 图片高度
 * @param {object} classMap 类别映射
 * @returns {string}
 */
export function toYOLOFormat(annotation, imageWidth, imageHeight, classMap = {}) {
    const classId = classMap[annotation.label] || 0;

    // 计算相对坐标
    const centerX = (annotation.x + annotation.width / 2) / imageWidth;
    const centerY = (annotation.y + annotation.height / 2) / imageHeight;
    const relWidth = annotation.width / imageWidth;
    const relHeight = annotation.height / imageHeight;

    if (annotation.type === AnnotationType.RECTANGLE) {
        return `${classId} ${centerX.toFixed(6)} ${centerY.toFixed(6)} ${relWidth.toFixed(6)} ${relHeight.toFixed(6)}`;
    }

    if (annotation.type === AnnotationType.ROTATED_RECTANGLE) {
        // YOLO格式的旋转框（OBB格式）
        const rotation = annotation.rotation || 0;
        return `${classId} ${centerX.toFixed(6)} ${centerY.toFixed(6)} ${relWidth.toFixed(6)} ${relHeight.toFixed(6)} ${rotation.toFixed(6)}`;
    }

    return '';
}

/**
 * 从YOLO格式解析标注数据
 * @param {string} yoloLine YOLO格式的行数据
 * @param {number} imageWidth 图片宽度
 * @param {number} imageHeight 图片高度
 * @param {object} classNames 类别名称映射
 * @returns {object|null}
 */
export function fromYOLOFormat(yoloLine, imageWidth, imageHeight, classNames = {}) {
    const parts = yoloLine.trim().split(/\s+/);

    if (parts.length < 5) {
        return null;
    }

    const classId = parseInt(parts[0]);
    const centerX = parseFloat(parts[1]) * imageWidth;
    const centerY = parseFloat(parts[2]) * imageHeight;
    const width = parseFloat(parts[3]) * imageWidth;
    const height = parseFloat(parts[4]) * imageHeight;

    const x = centerX - width / 2;
    const y = centerY - height / 2;

    const annotation = {
        id: Date.now() + Math.random(),
        type: parts.length > 5 ? AnnotationType.ROTATED_RECTANGLE : AnnotationType.RECTANGLE,
        x: Math.round(x),
        y: Math.round(y),
        width: Math.round(width),
        height: Math.round(height),
        label: classNames[classId] || `class_${classId}`,
        created: new Date().toISOString()
    };

    if (parts.length > 5) {
        annotation.rotation = parseFloat(parts[5]);
    }

    return validateAnnotation(annotation) ? annotation : null;
} 