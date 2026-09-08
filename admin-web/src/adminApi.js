import { expireAdminSession, getAdminSession } from "./session.js";

const API_BASE = import.meta.env.VITE_API_BASE || "http://127.0.0.1:8787";

let testTransport = null;

export function setAdminApiTransport(transport) {
  testTransport = transport;
}

export function resetAdminApi() {
  testTransport = null;
}

async function request(kind, extra = {}) {
  if (testTransport) {
    return testTransport({ kind, ...extra });
  }
  const { accessToken } = getAdminSession();
  if (!accessToken) throw new Error("需要先登录");
  const path = extra.riskPath ? `/v1/admin/${extra.riskPath}` : kind.startsWith('catalog') ? `/v1/admin/catalog/${extra.catalog}${extra.id ? `/${encodeURIComponent(extra.id)}` : ''}`
    : kind === 'content' ? `/v1/admin/content?${new URLSearchParams(extra.query ?? {})}`
    : kind === 'contentDetail' || kind === 'contentSave' ? `/v1/admin/content/${encodeURIComponent(extra.id)}`
    : kind === 'batchReview' ? '/v1/admin/reviews/batch'
    : kind === 'security' ? '/v1/admin/security'
    : kind === 'userDetail' ? `/v1/admin/users/${encodeURIComponent(extra.email)}`
    : kind === 'userAction' ? `/v1/admin/users/${encodeURIComponent(extra.email)}/actions`
    : kind === 'password' ? '/v1/admin/security/password'
    : kind === 'revokeSessions' ? '/v1/admin/security/sessions'
    : kind === 'getOAuth' ? '/v1/admin/oauth'
    : kind === 'putOAuth' ? `/v1/admin/oauth/${encodeURIComponent(extra.provider)}`
    : kind === "list"
      ? `/v1/admin/reviews?${new URLSearchParams(extra.query ?? {})}`
      : kind === "users"
        ? `/v1/admin/users?${new URLSearchParams(extra.query ?? {})}`
        : kind === "getSettings" || kind === "putSettings"
          ? "/v1/admin/settings"
          : `/v1/admin/publications/${encodeURIComponent(extra.id)}/${kind === "approve" ? "approve" : "reject"}`;
  const headers = { authorization: `Bearer ${accessToken}` };
  const init = { method: "GET", headers };
  if (extra.riskPath) {
    init.method = extra.method || 'GET';
    if (extra.config) { headers['content-type']='application/json'; init.body=JSON.stringify(extra.config); }
  } else if (['catalogCreate','catalogSave','catalogDelete'].includes(kind)) {
    init.method = kind === 'catalogCreate' ? 'POST' : kind === 'catalogSave' ? 'PUT' : 'DELETE';
    headers['content-type'] = 'application/json'; init.body = JSON.stringify(extra.config);
  } else if (kind === 'userAction' || kind === 'batchReview' || kind === 'reject') {
    init.method = 'POST';
    headers['content-type'] = 'application/json';
    init.body = JSON.stringify(kind === 'reject' ? { reason: extra.reason } : extra.config);
  } else if (kind === 'revokeSessions') {
    init.method = 'DELETE';
  } else if (kind === 'password' || kind === 'contentSave') {
    init.method = 'PUT';
    headers['content-type'] = 'application/json';
    init.body = JSON.stringify(extra.config);
  } else if (kind === "putSettings" || kind === 'putOAuth') {
    init.method = "PUT";
    headers["content-type"] = "application/json";
    init.body = JSON.stringify(kind === 'putOAuth' ? extra.config : { square_public: extra.square_public });
  } else if (kind !== 'catalog' && kind !== "list" && kind !== "users" && kind !== "getSettings" && kind !== 'getOAuth' && kind !== 'security' && kind !== 'userDetail' && kind !== 'content' && kind !== 'contentDetail') {
    init.method = "POST";
  }
  const response = await fetch(`${API_BASE}${path}`, init);
  if (getAdminSession().accessToken !== accessToken) throw new Error('会话已变化，已忽略旧请求');
  if (response.status === 403) throw new Error(['putOAuth','aiSave','mailSave','notificationSave','invite','userPasswordReset'].includes(kind) ? '请检查操作权限和当前密码，服务端未执行本次操作' : "当前账号没有此操作权限");
  if (response.status === 401) { expireAdminSession(accessToken); throw new Error('登录已失效，请重新登录'); }
  if (response.status === 429) throw new Error('操作过于频繁，请一分钟后重试');
  if (response.status === 409 && extra.riskPath?.startsWith('mock-billing/')) throw new Error('模拟未启用、版本冲突或请求已经处理。测试码若已生成不会再次返回明码，请查批次记录后停用旧批次再生成');
  if (response.status === 400 && extra.riskPath?.startsWith('mock-billing/')) throw new Error('请检查目标账号、原因、数量、额度和有效期');
  if (response.status === 400 && extra.riskPath === 'site') throw new Error('请检查名称长度、HTTPS 图片、邮箱和公告起止时间');
  if (response.status === 503 && ['invite','userPasswordReset'].includes(kind)) throw new Error('请先配置并启用邮件服务，验证邮件尚未发送');
  if (response.status === 409 && kind === 'invite') throw new Error('该邮箱已有账号，请在用户管理中调整权限，不能用邀请覆盖账号');
  if (response.status === 409 && kind === 'putOAuth') throw new Error('登录配置版本已变化，请刷新核对；草稿已保留');
  if (response.status === 400 && extra.riskPath?.startsWith('identity/')) throw new Error('请检查邮箱、角色和注册策略字段');
  if (response.status === 400 && extra.riskPath?.startsWith('mail/')) throw new Error('请检查 SMTP 公网主机、TLS 端口、邮箱和配置版本');
  if (response.status === 503 && kind.startsWith('notification')) throw new Error('通知渠道不可用；邮件通知需要先配置并启用 SMTP');
  if (response.status === 400 && kind.startsWith('notification')) throw new Error('请检查公网 HTTPS 地址、签名密钥、收件邮箱、阈值、额度及保留天数');
  if (response.status === 409 && kind.startsWith('notification')) throw new Error('配置版本已变化、渠道未启用或记录不可重试，请刷新核对；草稿已保留');
  if (extra.riskPath && response.status === 409) throw new Error('记录已变化或已结案，草稿已保留，请重新加载核对');
  if (response.status === 400 && kind === 'operations') throw new Error('请检查日期范围、筛选长度和分页参数；开始日期不能晚于结束日期');
  if (extra.riskPath && response.status === 400) throw new Error(extra.riskPath.startsWith('ai/') ? '请检查公网 HTTPS 地址、模型与 Skill 字段、密钥和样本长度' : '请检查必填原因、规则字段和负责人的有效权限');
  if (response.status === 409 && kind.startsWith('catalog')) throw new Error('字典冲突：名称或标识重复、版本已变化，或删除项仍有内容引用/子分类。草稿已保留，请刷新核对');
  if (response.status === 400 && kind.startsWith('catalog')) throw new Error('请检查字段格式和父分类；分类最多两级，不能形成循环');
  if (response.status === 409 && kind === 'contentSave') throw new Error('内容已被其他人修改，草稿已保留。请重新加载详情后再编辑');
  if (response.status === 409 && kind !== 'userAction') throw new Error("该投稿已有其他审核结果，请刷新审核列表");
  if (!response.ok) {
    const payload = await response.json().catch(() => ({}));
    throw new Error(payload.message || "管理请求失败，请稍后重试");
  }
  const result = await response.json();
  if (getAdminSession().accessToken !== accessToken) throw new Error('会话已变化，已忽略旧请求');
  return result;
}

export function listPendingPublications(query = {}) {
  return request("list", { query });
}
export const listCatalog = catalog => request('catalog', { catalog });
export const listReports = query => request('reports', {riskPath:`reports?${new URLSearchParams(query)}`});
export const getReport = id => request('reportDetail', {riskPath:`reports/${encodeURIComponent(id)}`});
export const updateReport = (id,config) => request('reportSave', {riskPath:`reports/${encodeURIComponent(id)}`,method:'PUT',config});
export const exportReports = query => request('reportsExport', {riskPath:`reports/export?${new URLSearchParams(query)}`});
export const getSafetyRules = () => request('rules', {riskPath:'safety-rules'});
export const saveSafetyRules = config => request('rulesSave', {riskPath:'safety-rules',method:'PUT',config});
export const testSafetyRules = config => request('rulesTest', {riskPath:'safety-rules/test',method:'POST',config});
export const getModerationPolicy = () => request('moderation', {riskPath:'moderation'});
export const getAiConfig = () => request('aiConfig', {riskPath:'ai/config'});
export const saveAiConfig = config => request('aiSave', {riskPath:'ai/config',method:'PUT',config});
export const testAiConfig = config => request('aiTest', {riskPath:'ai/test',method:'POST',config});
export const getAiHistory = () => request('aiHistory', {riskPath:'ai/history'});
export const getMailConfig = () => request('mailConfig', {riskPath:'mail/config'});
export const getNotificationConfig = () => request('notificationConfig',{riskPath:'notifications/config'});
export const saveNotificationConfig = config => request('notificationSave',{riskPath:'notifications/config',method:'PUT',config});
export const testNotification = config => request('notificationTest',{riskPath:'notifications/test',method:'POST',config});
export const listNotifications = query => request('notificationList',{riskPath:`notifications/deliveries?${new URLSearchParams(query)}`});
export const getNotification = id => request('notificationDetail',{riskPath:`notifications/deliveries/${encodeURIComponent(id)}`});
export const retryNotification = (id,config) => request('notificationRetry',{riskPath:`notifications/deliveries/${encodeURIComponent(id)}/retry`,method:'POST',config});
export const saveMailConfig = config => request('mailSave', {riskPath:'mail/config',method:'PUT',config});
export const testMail = config => request('mailTest', {riskPath:'mail/test',method:'POST',config});
export const listMail = query => request('mailList', {riskPath:`mail/deliveries?${new URLSearchParams(query)}`});
export const getMail = id => request('mailDetail', {riskPath:`mail/deliveries/${encodeURIComponent(id)}`});
export const retryMail = (id,config) => request('mailRetry', {riskPath:`mail/deliveries/${encodeURIComponent(id)}/retry`,method:'POST',config});
export const getIdentityPolicy = () => request('identityPolicy', {riskPath:'identity/policy'});
export const getSiteConfig = () => request('siteConfig', {riskPath:'site'});
export const getOperations = (page,query={}) => request('operations', {riskPath:`${page}?${new URLSearchParams(query)}`});
export const listMockBilling = (kind,query) => request('mockList', {riskPath:`mock-billing/${kind}?${new URLSearchParams(query)}`});
export const changeMockBilling = config => request('mockChange', {riskPath:'mock-billing/actions',method:'POST',config});
export const createMockBatch = config => request('mockBatchCreate', {riskPath:'mock-billing/batches',method:'POST',config});
export const getMockBatch = id => request('mockBatchDetail', {riskPath:`mock-billing/batches/${encodeURIComponent(id)}`});
export const updateMockBatch = (id,config) => request('mockBatchUpdate', {riskPath:`mock-billing/batches/${encodeURIComponent(id)}`,method:'PUT',config});
export const saveSiteConfig = config => request('siteSave', {riskPath:'site',method:'PUT',config});
export const saveIdentityPolicy = config => request('identitySave', {riskPath:'identity/policy',method:'PUT',config});
export const listInvitations = offset => request('invitations', {riskPath:`identity/invitations?offset=${offset}`});
export const inviteAdmin = config => request('invite', {riskPath:'identity/invitations',method:'POST',config});
export const revokeInvitation = (id,config) => request('inviteRevoke', {riskPath:`identity/invitations/${encodeURIComponent(id)}/revoke`,method:'POST',config});
export const resetUserPassword = (email,config) => request('userPasswordReset', {riskPath:`users/${encodeURIComponent(email)}/password-reset`,method:'POST',config});
export const saveModerationPolicy = config => request('moderationSave', {riskPath:'moderation',method:'PUT',config});
export const saveCatalog = (catalog, config, editing) => request(editing ? 'catalogSave' : 'catalogCreate', { catalog, ...(editing ? { id:config.id } : {}), config });
export const deleteCatalog = (catalog, id, revision) => request('catalogDelete', { catalog, id, config:{revision} });
export const migrateCatalog = (catalog,id,config) => request('catalogMigrate', {riskPath:`catalog/${catalog}/${encodeURIComponent(id)}/migrate`,method:'POST',config});

export function approvePublication(id) {
  return request("approve", { id });
}

export function rejectPublication(id, reason) {
  return request("reject", { id, reason });
}
export function batchReviewPublications(config) { return request('batchReview', { config }); }
export function listAdminContent(query = {}) { return request('content', { query }); }
export function getAdminContent(id) { return request('contentDetail', { id }); }
export function saveAdminContent(id, config) { return request('contentSave', { id, config }); }

export function listAdminUsers(query = {}) {
  return request("users", { query });
}
export function getAdminUser(email) { return request('userDetail', { email }); }
export async function manageAdminUser(email, config) {
  const result = await request('userAction', { email, config });
  if (result?.updated !== true) throw new Error('服务端未确认操作完成，请刷新检查');
  return result;
}

export function getAdminSettings() {
  return request("getSettings");
}

export function putAdminSettings(squarePublic) {
  return request("putSettings", { square_public: squarePublic });
}

export function getOAuthSettings() { return request('getOAuth'); }
export function verifyOAuthSettings(provider) { return request('verifyOAuth',{riskPath:`oauth/${provider}/verify`,method:'POST',config:{}}); }
export function putOAuthSettings(provider, config) { return request('putOAuth', { provider, config }); }
export function getAccountSecurity() { return request('security'); }
async function securityWrite(kind, extra) {
  const result = await request(kind, extra);
  if (result?.signed_out !== true) throw new Error('服务端未确认完成，请重新加载账号安全信息');
  return result;
}
export function changeAdminPassword(currentPassword, newPassword) { return securityWrite('password', { config: { current_password: currentPassword, new_password: newPassword } }); }
export function revokeAdminSessions() { return securityWrite('revokeSessions'); }
