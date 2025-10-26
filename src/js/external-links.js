/**
 * 外部链接处理模块
 * 使用 Tauri opener 插件正确打开外部链接
 */

/**
 * 打开外部链接的函数（全局函数）
 * @param {string} url - 要打开的 URL  
 * @param {string} openWith - 可选，指定用哪个应用打开
 */
window.openExternalLink = async function(url, openWith = null) {
    try {
        console.log('正在打开外部链接:', url);
        
        // 方法1: 尝试使用 Tauri 的 shell open 命令
        if (window.__TAURI__ && window.__TAURI__.shell) {
            try {
                await window.__TAURI__.shell.open(url);
                console.log('外部链接打开成功 (shell.open):', url);
                return;
            } catch (shellError) {
                console.warn('shell.open 调用失败:', shellError);
            }
        }
        
        // 方法2: 尝试使用 Tauri 的 invoke 方法
        if (window.__TAURI__ && window.__TAURI__.core) {
            try {
                const params = openWith ? { url: url, with: openWith } : { url: url };
                await window.__TAURI__.core.invoke('plugin:opener|open_url', params);
                console.log('外部链接打开成功 (invoke):', url);
                return;
            } catch (invokeError) {
                console.warn('invoke opener 调用失败:', invokeError);
            }
        }
        
        // 方法3: 回退到传统的 window.open 方法
        console.log('使用传统方法打开链接');
        const newWindow = window.open(url, '_blank', 'noopener,noreferrer');
        if (newWindow && !newWindow.closed) {
            return;
        }
        
        // 如果所有方法都失败，抛出错误进入后备方案
        throw new Error('所有打开方法都失败');
        
    } catch (error) {
        console.error('打开外部链接失败:', error);
        
        // 最后的后备方案：复制到剪贴板
        try {
            await navigator.clipboard.writeText(url);
            if (window.showMessage) {
                window.showMessage(`📋 无法自动打开链接，已复制到剪贴板:\n${url}\n\n请手动粘贴到浏览器中打开。`);
            } else {
                alert(`无法自动打开链接，已复制到剪贴板:\n${url}\n\n请手动粘贴到浏览器中打开。`);
            }
        } catch (clipboardError) {
            console.error('复制到剪贴板也失败了:', clipboardError);
            if (window.showMessage) {
                window.showMessage(`❌ 无法打开链接: ${url}\n\n请手动复制此链接到浏览器中打开。`);
            } else {
                alert(`无法打开链接: ${url}\n\n请手动复制此链接到浏览器中打开。`);
            }
        }
    }
};

// 当 DOM 加载完成后输出日志
document.addEventListener('DOMContentLoaded', () => {
    console.log('外部链接处理器初始化完成');
    console.log('可用的 Tauri API:', {
        hasTauri: !!window.__TAURI__,
        hasShell: !!(window.__TAURI__ && window.__TAURI__.shell),
        hasCore: !!(window.__TAURI__ && window.__TAURI__.core)
    });
});