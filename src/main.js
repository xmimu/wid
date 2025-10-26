const { invoke } = window.__TAURI__.core;

// 配置存储对象
const config = {
  wwise: {
    projPath: localStorage.getItem('wwise_proj_path') || ''
  },
  waapi: {
    host: localStorage.getItem('waapi_host') || '127.0.0.1',
    port: localStorage.getItem('waapi_port') || '8080'
  },
  bank: {
    dirPath: localStorage.getItem('bank_dir_path') || ''
  }
};

// 分页数据存储
const paginationData = {
  wwise: { currentPage: 1, pageSize: 20, totalResults: [] },
  waapi: { currentPage: 1, pageSize: 20, totalResults: [] },
  bank: { currentPage: 1, pageSize: 20, totalResults: [] }
};

// 保存配置到本地存储
function saveConfig() {
  localStorage.setItem('wwise_proj_path', config.wwise.projPath);
  localStorage.setItem('waapi_host', config.waapi.host);
  localStorage.setItem('waapi_port', config.waapi.port);
  localStorage.setItem('bank_dir_path', config.bank.dirPath);
}

// 获取当前激活的标签页
function getCurrentTab() {
  const activeTab = document.querySelector('.nav-link.active');
  if (activeTab.id === 'wwise-tab') return 'wwise';
  if (activeTab.id === 'waapi-tab') return 'waapi';
  if (activeTab.id === 'bank-tab') return 'bank';
  return null;
}

// 根据标签页获取结果表格body
function getResultsTableBody(tab) {
  return document.querySelector(`.resultsTableBody[data-tab="${tab}"]`);
}

// 根据标签页获取结果计数
function getResultCount(tab) {
  return document.querySelector(`.resultCount[data-tab="${tab}"]`);
}

// 打开文件夹浏览器（Wwise项目）
async function browseProjPath() {
  try {
    console.log("点击了浏览按钮");
    
    // 检查 Tauri API 是否可用
    if (!window.__TAURI__) {
      throw new Error("Tauri API 未加载");
    }
    
    // 使用 Tauri 2 的 dialog API
    const selected = await window.__TAURI__.dialog.open({
      directory: true,
      multiple: false,
      title: "选择 Wwise 工程目录（包含 .wproj 文件）"
    });
    
    console.log("选择的目录:", selected);
    
    if (selected) {
      // 验证目录
      try {
        const isValid = await invoke("validate_wwise_directory", { path: selected });
        if (isValid) {
          config.wwise.projPath = selected;
          document.querySelector('#wwiseProjPath').value = selected;
          saveConfig();
          checkConfigAndShowAlert();
          showMessage("✅ 目录验证成功！");
        }
      } catch (validationError) {
        showMessage("❌ " + validationError);
        console.error("目录验证失败:", validationError);
      }
    }
  } catch (error) {
    console.error("选择目录出错:", error);
    showMessage("❌ 选择目录失败: " + error);
  }
}

// 打开文件夹浏览器（Bank目录）
async function browseBankPath() {
  try {
    console.log("点击了Bank浏览按钮");
    
    // 检查 Tauri API 是否可用
    if (!window.__TAURI__) {
      throw new Error("Tauri API 未加载");
    }
    
    // 使用 Tauri 2 的 dialog API
    const selected = await window.__TAURI__.dialog.open({
      directory: true,
      multiple: false,
      title: "选择 Bank 目录（包含 SoundbanksInfo.xml 或 .json）"
    });
    
    console.log("选择的目录:", selected);
    
    if (selected) {
      // 验证目录
      try {
        const isValid = await invoke("validate_bank_directory", { path: selected });
        if (isValid) {
          config.bank.dirPath = selected;
          document.querySelector('#bankDirPath').value = selected;
          saveConfig();
          checkConfigAndShowAlert();
          showMessage("✅ 目录验证成功！");
        }
      } catch (validationError) {
        showMessage("❌ " + validationError);
        console.error("目录验证失败:", validationError);
      }
    }
  } catch (error) {
    console.error("选择目录出错:", error);
    showMessage("❌ 选择目录失败: " + error);
  }
}

// 测试WAAPI连接（调用 waapi-query.js 中的函数）
async function testWaapiConnectionHandler() {
  const host = document.querySelector('#waapiHost').value;
  const port = document.querySelector('#waapiPort').value;
  
  if (!host || !port) {
    showMessage("请输入 Host 和 Port");
    return;
  }

  config.waapi.host = host;
  config.waapi.port = port;
  saveConfig();

  try {
    const message = await testWaapiConnection(host, port);
    showMessage(message);
  } catch (error) {
    showMessage("❌ " + error);
    console.error("WAAPI 连接测试失败:", error);
  }
}


// 执行搜索
async function performSearch(tab) {
  // 使用共享的输入框和复选框
  const idInput = document.querySelector('#sharedIdInput');
  const typeCheckboxes = document.querySelectorAll('#sharedGuidCheckbox, #sharedShortIdCheckbox, #sharedMediaIdCheckbox');
  const resultsTableBody = getResultsTableBody(tab);
  const resultCount = getResultCount(tab);
  
  // 检查配置
  if (tab === 'wwise' && !config.wwise.projPath) {
    showMessage("请先设置 Wwise 工程目录");
    return;
  }
  if (tab === 'waapi' && (!config.waapi.host || !config.waapi.port)) {
    showMessage("请先设置 WAAPI 连接信息");
    return;
  }
  if (tab === 'bank' && !config.bank.dirPath) {
    showMessage("请先设置 Bank 目录");
    return;
  }
  
  const searchValue = idInput.value.trim();
  const selectedTypes = [];
  
  typeCheckboxes.forEach(checkbox => {
    if (checkbox.checked) {
      selectedTypes.push(checkbox.value);
    }
  });

  if (!searchValue) {
    showMessage("请输入查询条件");
    return;
  }

  if (selectedTypes.length === 0) {
    showMessage("请至少选择一种ID类型");
    return;
  }

  try {
    let results = [];
    
    // 根据不同标签页调用相应的API
    if (tab === 'wwise') {
      results = await invoke("search_wwise_project", { 
        directory: config.wwise.projPath, 
        idString: searchValue, 
        idTypes: selectedTypes 
      });
    } else if (tab === 'waapi') {
      // 使用 JavaScript WAAPI 查询（调用 waapi-query.js）
      results = await searchWithWAAPI(
        config.waapi.host, 
        config.waapi.port, 
        searchValue, 
        selectedTypes
      );
      // WAAPI 结果已经是新格式，不需要转换
    } else if (tab === 'bank') {
      results = await invoke("search_bank_directory", { 
        directory: config.bank.dirPath, 
        idString: searchValue, 
        idTypes: selectedTypes 
      });
    }

    // 保存结果并显示第一页
    paginationData[tab].totalResults = results;
    paginationData[tab].currentPage = 1;
    displayResultsWithPagination(tab);
  } catch (error) {
    showMessage("搜索失败: " + error);
    console.error("搜索错误:", error);
  }
}

// 显示搜索结果（带分页）
function displayResultsWithPagination(tab) {
  const resultsTableBody = getResultsTableBody(tab);
  const resultCount = getResultCount(tab);
  const pageInfo = document.querySelector(`.pageInfo[data-tab="${tab}"]`);
  const pagination = document.querySelector(`.pagination[data-tab="${tab}"]`);
  
  const data = paginationData[tab];
  const totalResults = data.totalResults;
  const currentPage = data.currentPage;
  const pageSize = data.pageSize;
  
  resultsTableBody.innerHTML = '';
  
  if (totalResults.length === 0) {
    resultsTableBody.innerHTML = `
      <tr>
        <td colspan="5" class="text-center text-muted">
          未找到匹配的结果
        </td>
      </tr>
    `;
    resultCount.textContent = '0';
    pageInfo.textContent = '';
    pagination.innerHTML = '';
    return;
  }

  // 计算分页
  const totalPages = Math.ceil(totalResults.length / pageSize);
  const startIndex = (currentPage - 1) * pageSize;
  const endIndex = Math.min(startIndex + pageSize, totalResults.length);
  const pageResults = totalResults.slice(startIndex, endIndex);

  // 显示当前页结果
  pageResults.forEach(item => {
    const row = document.createElement('tr');
    row.innerHTML = `
      <td>${item.name || ''}</td>
      <td>${item.object_type || ''}</td>
      <td><code>${item.guid || ''}</code></td>
      <td>${item.short_id || ''}</td>
      <td>${item.media_id || ''}</td>
    `;
    resultsTableBody.appendChild(row);
  });

  // 更新计数
  resultCount.textContent = totalResults.length;
  pageInfo.textContent = ` (第 ${currentPage}/${totalPages} 页)`;

  // 生成分页控件
  renderPagination(tab, currentPage, totalPages);
}

// 渲染分页控件
function renderPagination(tab, currentPage, totalPages) {
  const pagination = document.querySelector(`.pagination[data-tab="${tab}"]`);
  pagination.innerHTML = '';

  if (totalPages <= 1) {
    return;
  }

  // 上一页按钮
  const prevLi = document.createElement('li');
  prevLi.className = `page-item ${currentPage === 1 ? 'disabled' : ''}`;
  prevLi.innerHTML = `<a class="page-link" href="#" tabindex="-1">上一页</a>`;
  if (currentPage > 1) {
    prevLi.addEventListener('click', (e) => {
      e.preventDefault();
      paginationData[tab].currentPage--;
      displayResultsWithPagination(tab);
    });
  }
  pagination.appendChild(prevLi);

  // 页码按钮
  const maxButtons = 5;
  let startPage = Math.max(1, currentPage - Math.floor(maxButtons / 2));
  let endPage = Math.min(totalPages, startPage + maxButtons - 1);
  
  if (endPage - startPage < maxButtons - 1) {
    startPage = Math.max(1, endPage - maxButtons + 1);
  }

  for (let i = startPage; i <= endPage; i++) {
    const pageLi = document.createElement('li');
    pageLi.className = `page-item ${i === currentPage ? 'active' : ''}`;
    pageLi.innerHTML = `<a class="page-link" href="#">${i}</a>`;
    if (i !== currentPage) {
      pageLi.addEventListener('click', (e) => {
        e.preventDefault();
        paginationData[tab].currentPage = i;
        displayResultsWithPagination(tab);
      });
    }
    pagination.appendChild(pageLi);
  }

  // 下一页按钮
  const nextLi = document.createElement('li');
  nextLi.className = `page-item ${currentPage === totalPages ? 'disabled' : ''}`;
  nextLi.innerHTML = `<a class="page-link" href="#">下一页</a>`;
  if (currentPage < totalPages) {
    nextLi.addEventListener('click', (e) => {
      e.preventDefault();
      paginationData[tab].currentPage++;
      displayResultsWithPagination(tab);
    });
  }
  pagination.appendChild(nextLi);
}

// 显示搜索结果（旧版本，已废弃）
function displayResults(results, resultsTableBody, resultCount) {
  resultsTableBody.innerHTML = '';
  
  if (results.length === 0) {
    resultsTableBody.innerHTML = `
      <tr>
        <td colspan="2" class="text-center text-muted">
          未找到匹配的结果
        </td>
      </tr>
    `;
    resultCount.textContent = '0';
    return;
  }

  results.forEach(item => {
    const row = document.createElement('tr');
    row.innerHTML = `
      <td>${item.name}</td>
      <td><code>${item.id}</code></td>
    `;
    resultsTableBody.appendChild(row);
  });

  resultCount.textContent = results.length;
}

// 显示提示信息
function showMessage(message) {
  alert(message);
}

// 清空表单和结果
function clearAll(tab) {
  const idInput = document.querySelector('#sharedIdInput');
  const resultsTableBody = getResultsTableBody(tab);
  const resultCount = getResultCount(tab);
  const pageInfo = document.querySelector(`.pageInfo[data-tab="${tab}"]`);
  const pagination = document.querySelector(`.pagination[data-tab="${tab}"]`);
  
  idInput.value = '';
  resultsTableBody.innerHTML = `
    <tr>
      <td colspan="5" class="text-center text-muted">
        请输入ID并点击搜索
      </td>
    </tr>
  `;
  resultCount.textContent = '0';
  pageInfo.textContent = '';
  pagination.innerHTML = '';
  
  // 清空分页数据
  paginationData[tab].totalResults = [];
  paginationData[tab].currentPage = 1;
  
  idInput.focus();
}

// 检查配置并显示提示
function checkConfigAndShowAlert() {
  // 检查 Wwise 配置
  const wwiseAlert = document.querySelector('#wwiseConfigAlert');
  if (!config.wwise.projPath) {
    wwiseAlert.style.display = 'block';
  } else {
    wwiseAlert.style.display = 'none';
  }

  // 检查 WAAPI 配置
  const waapiAlert = document.querySelector('#waapiConfigAlert');
  if (!config.waapi.host || !config.waapi.port) {
    waapiAlert.style.display = 'block';
  } else {
    waapiAlert.style.display = 'none';
  }

  // 检查 Bank 配置
  const bankAlert = document.querySelector('#bankConfigAlert');
  if (!config.bank.dirPath) {
    bankAlert.style.display = 'block';
  } else {
    bankAlert.style.display = 'none';
  }
}

// 切换搜索区域的显示/隐藏
function toggleSearchArea() {
  const currentTab = getCurrentTab();
  const searchArea = document.querySelector('#sharedSearchArea');
  
  // 如果是配置标签页，隐藏搜索区域；否则显示
  if (currentTab === null) {
    searchArea.style.display = 'none';
  } else {
    searchArea.style.display = 'block';
  }
}

// 初始化
window.addEventListener("DOMContentLoaded", () => {
  // 初始化配置显示
  document.querySelector('#wwiseProjPath').value = config.wwise.projPath;
  document.querySelector('#waapiHost').value = config.waapi.host;
  document.querySelector('#waapiPort').value = config.waapi.port;
  document.querySelector('#bankDirPath').value = config.bank.dirPath;

  // 初始检查配置
  checkConfigAndShowAlert();

  // 初始检查搜索区域显示状态
  toggleSearchArea();

  // 监听标签切换事件
  const tabButtons = document.querySelectorAll('button[data-bs-toggle="tab"]');
  tabButtons.forEach(button => {
    button.addEventListener('shown.bs.tab', () => {
      checkConfigAndShowAlert();
      toggleSearchArea();
    });
  });

  // 配置输入框监听（自动保存）
  document.querySelector('#wwiseProjPath').addEventListener('change', (e) => {
    config.wwise.projPath = e.target.value;
    saveConfig();
    checkConfigAndShowAlert();
  });
  document.querySelector('#waapiHost').addEventListener('change', (e) => {
    config.waapi.host = e.target.value;
    saveConfig();
    checkConfigAndShowAlert();
  });
  document.querySelector('#waapiPort').addEventListener('change', (e) => {
    config.waapi.port = e.target.value;
    saveConfig();
    checkConfigAndShowAlert();
  });
  document.querySelector('#bankDirPath').addEventListener('change', (e) => {
    config.bank.dirPath = e.target.value;
    saveConfig();
    checkConfigAndShowAlert();
  });

  // 浏览按钮
  document.querySelector('#wwiseBrowseBtn').addEventListener('click', browseProjPath);
  document.querySelector('#bankBrowseBtn').addEventListener('click', browseBankPath);
  document.querySelector('#waapiTestBtn').addEventListener('click', testWaapiConnectionHandler);

  // 绑定共享搜索按钮事件
  document.querySelector('#sharedSearchBtn').addEventListener('click', () => {
    const tab = getCurrentTab();
    if (tab) {
      performSearch(tab);
    }
  });

  // 绑定共享清空按钮事件
  document.querySelector('#sharedClearBtn').addEventListener('click', () => {
    const tab = getCurrentTab();
    if (tab) {
      clearAll(tab);
    }
  });

  // 按Enter键搜索
  document.querySelector('#sharedIdInput').addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
      const tab = getCurrentTab();
      if (tab) {
        performSearch(tab);
      }
    }
  });
});
