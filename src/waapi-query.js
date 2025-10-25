// WAAPI 查询模块 - 使用 WAQL

/**
 * 测试 WAAPI 连接
 * @param {string} host - WAAPI 服务器地址
 * @param {string} port - WAAPI 服务器端口
 * @returns {Promise<string>} 连接结果信息
 */
async function testWaapiConnection(host, port) {
  return new Promise((resolve, reject) => {
    const url = `ws://${host}:${port}/waapi`;
    const connection = new autobahn.Connection({
      url: url,
      realm: 'realm1',
      protocols: ['wamp.2.json']
    });

    let timeoutId = setTimeout(() => {
      connection.close();
      reject("连接超时（10秒）。请确保 Wwise 已启动并开启了 WAAPI。");
    }, 10000);

    connection.onopen = async (session) => {
      clearTimeout(timeoutId);
      try {
        // 调用 ak.wwise.core.getInfo 获取 Wwise 信息
        const result = await session.call('ak.wwise.core.getInfo');
        const version = result.kwargs.version?.displayName || 'Unknown';
        const platform = result.kwargs.platform?.basePlatform || result.kwargs.platform || 'Unknown';
        const displayName = result.kwargs.displayName || 'Wwise';
        
        const message = `✅ 连接成功！\n\n应用: ${displayName}\nWwise 版本: ${version}\n平台: ${platform}`;
        connection.close();
        resolve(message);
      } catch (error) {
        connection.close();
        reject("调用 WAAPI 失败: " + (error.error || error));
      }
    };

    connection.onclose = (reason, details) => {
      clearTimeout(timeoutId);
      if (reason === 'closed') {
        return; // 正常关闭
      }
      if (reason === 'unreachable') {
        reject("无法连接到 WAAPI。请检查 Wwise 是否已启动，以及 Host 和 Port 是否正确。");
      } else {
        reject(`连接关闭: ${reason}`);
      }
    };

    connection.open();
  });
}

/**
 * 使用 WAQL 查询 Wwise 对象
 * @param {object} session - autobahn session 对象
 * @param {string} idString - 要查询的 ID 字符串
 * @param {Array<string>} idTypes - ID 类型数组 ['GUID', 'ShortID', 'MediaID']
 * @returns {Promise<Array>} 查询结果数组
 */
async function searchWaapiByWaql(session, idString, idTypes) {
  const results = [];
  
  for (const idType of idTypes) {
    try {
      let waql = '';
      
      if (idType === 'GUID') {
        // WAQL: 查询 GUID（支持完整 GUID 或部分匹配）
        // 如果包含 {}, 使用精确查询；否则使用模糊查询
        if (idString.includes('{') && idString.includes('}')) {
          waql = `$ "${idString}"`;
        } else if (idString.includes('-')) {
          // 带 - 的 GUID，补充 {}
          waql = `$ "{${idString}}"`;
        } else {
          // 部分 GUID，使用通配符
          waql = `from object "*" where id =* "*${idString}*"`;
        }
      } else if (idType === 'ShortID') {
        // WAQL: 查询 ShortID
        waql = `from type Sound, SoundBank, Event, MusicSegment, DialogueEvent, Actor, AuxBus, RandomOrSequenceContainer, SwitchContainer, BlendContainer where shortId = ${idString}`;
      } else if (idType === 'MediaID') {
        // WAQL: 查询包含特定 MediaID 的源文件
        waql = `from type Sound where mediaId = ${idString}`;
      }
      
      if (!waql) continue;
      
      // 构建参数
      const getParamArgs = {
        waql: waql
      };
      
      const options = {
        return: ['id', 'name', 'type', 'shortId', 'path']
      };
      
      // 调用 ak.wwise.core.object.get with WAQL
      const result = await session.call('ak.wwise.core.object.get', [], getParamArgs, options);
      
      if (result.kwargs.return && result.kwargs.return.length > 0) {
        result.kwargs.return.forEach(obj => {
          results.push({
            name: `${obj.name} (${obj.type}) - ${idType}`,
            id: obj.id,
            shortId: obj.shortId || 'N/A',
            path: obj.path
          });
        });
      }
    } catch (error) {
      console.error(`WAQL 查询 ${idType} 失败:`, error);
    }
  }
  
  return results;
}

/**
 * 使用 WAAPI 执行搜索（统一接口）
 * @param {string} host - WAAPI 服务器地址
 * @param {string} port - WAAPI 服务器端口
 * @param {string} idString - 要查询的 ID 字符串
 * @param {Array<string>} idTypes - ID 类型数组 ['GUID', 'ShortID', 'MediaID']
 * @returns {Promise<Array>} 查询结果数组
 */
async function searchWithWAAPI(host, port, idString, idTypes) {
  return new Promise((resolve, reject) => {
    const url = `ws://${host}:${port}/waapi`;
    
    const connection = new autobahn.Connection({
      url: url,
      realm: 'realm1',
      protocols: ['wamp.2.json']
    });

    let timeoutId = setTimeout(() => {
      connection.close();
      reject("连接超时（10秒）");
    }, 10000);

    connection.onopen = async (session) => {
      clearTimeout(timeoutId);
      try {
        // 使用 WAQL 查询
        const results = await searchWaapiByWaql(session, idString, idTypes);
        
        // 去重（根据 id）
        const uniqueResults = Array.from(
          new Map(results.map(item => [item.id, item])).values()
        );
        
        connection.close();
        resolve(uniqueResults);
      } catch (error) {
        connection.close();
        reject(error.error || error.message || error);
      }
    };

    connection.onclose = (reason, details) => {
      clearTimeout(timeoutId);
      if (reason === 'closed') {
        return; // 正常关闭
      }
      reject(`连接失败: ${reason}`);
    };

    connection.open();
  });
}
