<template>
  <div class="app-shell" :inert="session.loggedIn && !profileReady ? '' : undefined" :class="{ 'sidebar-collapsed': sidebarCollapsed, 'sidebar-resizing': sidebarDrag !== null }" :style="{ '--sidebar-width': `${sidebarWidth}px` }">
    <header
      v-show="!settingsOpen || Boolean(loginReason)"
      :inert="globalSearchOpen ? '' : undefined"
      data-region="titlebar"
      class="titlebar"
      :class="{ 'host-mac': host === 'macos' }"
      :style="{ '--traffic-light-inset': `${trafficInset}px` }"
      data-tauri-drag-region
    >
      <div class="titlebar-left" data-tauri-drag-region>
        <button type="button" class="sidebar-toggle" data-testid="toggle-sidebar" :aria-label="sidebarCollapsed ? '展开侧栏' : '收起侧栏'" :aria-expanded="!sidebarCollapsed" aria-controls="workbench-sidebar" @click="sidebarCollapsed = !sidebarCollapsed"><AppIcon name="sidebar" /></button>
      </div>
      <div class="titlebar-center" data-tauri-drag-region>
        <AppIcon :name="space === 'local' ? 'folder' : 'square'" /><span data-tauri-drag-region>{{ taskTitle || locationLabel }}</span>
      </div>
      <div class="titlebar-right">
        <button type="button" class="title-tool" data-testid="titlebar-search" :title="`全局搜索 ${globalSearchShortcutLabel}`" :disabled="globalSearchBlocked" @click="openGlobalSearch">
          <AppIcon name="search" /><span>全局搜索</span><kbd>{{ globalSearchShortcutLabel }}</kbd>
        </button>
      </div>
    </header>

    <div v-show="!settingsOpen || Boolean(loginReason)" class="workspace" :inert="globalSearchOpen ? '' : undefined">
      <aside v-show="!sidebarCollapsed" id="workbench-sidebar" data-region="sidebar" class="sidebar" @click.capture="guardSidebarNavigation">
        <div class="sidebar-brand-row">
          <span class="brand-name">{{ t("brand") }}</span>
        </div>
        <p class="nav-group-label">提示词</p>
        <div class="space-switch" role="tablist" aria-label="提示词空间">
          <button
            type="button"
            class="space-tab"
            data-space="square"
            role="tab"
            :aria-selected="contentKind === 'prompts' && space === 'square'"
            :class="{ active: contentKind === 'prompts' && space === 'square' }"
            @click="openSquare"
          >
            <span class="nav-icon"><AppIcon name="square" /></span><span>{{ t("square") }}</span>
          </button>
          <button
            type="button"
            class="space-tab"
            data-space="local"
            role="tab"
            :aria-selected="contentKind === 'prompts' && space === 'local'"
            :class="{ active: contentKind === 'prompts' && space === 'local' }"
            @click="openLocal"
          >
            <span class="nav-icon"><AppIcon name="library" /></span><span>{{ t("local") }}</span>
          </button>
        </div>

        <p class="nav-group-label">Skills</p>
        <div class="space-switch" role="tablist" aria-label="Skills 空间">
          <button type="button" class="space-tab" data-space="skills-square" role="tab" :aria-selected="contentKind === 'skills' && skillsMode === 'square'" :class="{active:contentKind === 'skills' && skillsMode === 'square'}" @click="openSkills('square')"><span class="nav-icon"><AppIcon name="square"/></span><span>Skill 广场</span></button>
          <button type="button" class="space-tab" data-space="skills-local" role="tab" :aria-selected="contentKind === 'skills' && skillsMode === 'local'" :class="{active:contentKind === 'skills' && skillsMode === 'local'}" @click="openSkills('local')"><span class="nav-icon"><AppIcon name="skills"/></span><span>本机 Skills</span></button>
        </div>
        <div v-show="contentKind === 'prompts'" class="sidebar-toolbar">
          <span>{{ space === "local" ? t("myCategories") : t("exploreCategories") }}</span>
          <div>
            <button type="button" class="mini-button category-collapse" title="全部折叠" @click="collapseAll">折叠</button>
            <button v-if="space === 'local'" type="button" class="mini-button" data-testid="add-category" title="新建分类" aria-label="新建分类" @click="startAddCategory">＋</button>
          </div>
        </div>

        <nav v-show="contentKind === 'prompts'" ref="categoryTree" class="category-tree" aria-label="提示词分类">
          <button
            type="button"
            class="tree-row"
            :class="{ active: !selectedId }"
            @click="selectCategory(null)"
          >
            <span class="chevron ghost">›</span>
            <span class="tree-icon warm"><AppIcon name="square" /></span>
            <span>{{ t("allPrompts") }}</span>
            <span class="tree-count" :title="space === 'square' ? '全广场公开条目数，不受当前筛选影响' : undefined">{{ space === 'local' ? allLocalItems.length : squareCategoryTotal ?? '—' }}</span>
          </button>
          <button v-if="space === 'local'" type="button" class="tree-row" data-testid="uncategorized"
            :class="{ active: selectedId === '__uncategorized__' }" @click="selectCategory('__uncategorized__')">
            <span class="chevron ghost">›</span>
            <span class="tree-icon"><AppIcon name="folder" /></span>
            <span>{{ uiLanguage === 'en' ? 'Uncategorized' : '未分类' }}</span>
            <span class="tree-count">{{ categoryCount('__uncategorized__') }}</span>
          </button>
          <div v-for="group in visibleCategoryGroups" :key="group.id" class="tree-group" :class="{ open: group.open }">
            <div class="tree-child-row">
            <button type="button" class="tree-expand" :aria-label="`${group.open ? '收起' : '展开'} ${group.name}`" :aria-expanded="group.open" @click="toggleGroup(group)"><span class="chevron">›</span></button>
            <button type="button" class="tree-row tree-parent" :class="{ active: selectedId === group.id }" @click="selectCategory(group.id)">
              <span class="tree-icon" :class="group.tone" :style="space === 'square' && group.color ? { color:group.color } : undefined"><AppIcon :name="space === 'square' && group.icon ? group.icon : group.tone" /></span>
              <span>{{ group.name }}</span>
              <span class="tree-count" :title="space === 'square' ? '公开条目数（含子分类），不受当前筛选影响' : undefined">{{ categoryCount(group.id) }}</span>
            </button>
            <button v-if="space === 'local' && !group.is_system" type="button" class="category-delete"
              :aria-label="`删除分类 ${group.name}`" :title="`删除分类 ${group.name}`" @click="startDeleteCategory(group)">×</button>
            </div>
            <div class="tree-children">
              <div v-for="child in group.children" :key="child.id" v-show="space === 'local' || child.is_system" class="tree-child-row">
              <button
                type="button"
                class="tree-row child"
                :class="{ active: selectedId === child.id }"
                @click="selectCategory(child.id)"
              >
                <span>{{ child.name }}</span>
                <span class="tree-count" :title="space === 'square' ? '公开条目数，不受当前筛选影响' : undefined">{{ categoryCount(child.id) }}</span>
              </button>
              <button v-if="space === 'local' && !child.is_system" type="button" class="category-delete"
                :aria-label="`删除分类 ${child.name}`" :title="`删除分类 ${child.name}`" @click="startDeleteCategory(child)">×</button>
              </div>
            </div>
          </div>
        </nav>

        <div v-show="contentKind === 'skills'" class="sidebar-toolbar"><span>Skill 分类</span><span>自动分类</span></div>
        <nav v-show="contentKind === 'skills'" class="category-tree skills-category-tree" aria-label="Skill 分类">
          <p class="skills-category-scope">{{ skillsMode === 'square' ? '数量仅含当前已加载来源' : '数量按 Skill 计，合并同源安装' }}</p>
          <button v-for="category in skillCategories" :key="category.id" type="button" class="tree-row"
            :data-skill-category="category.id || 'all'" :class="{active:skillsCategories[skillsMode] === category.id}"
            :aria-pressed="skillsCategories[skillsMode] === category.id" :disabled="skillsBusy" @click="selectSkillCategory(category.id)">
            <span class="tree-icon"><AppIcon :name="category.icon"/></span><span>{{ category.name }}</span>
            <span class="tree-count" :title="skillsCategorySummary[skillsMode]?.scope">{{ skillsCategorySummary[skillsMode]?.ready ? skillsCategorySummary[skillsMode].counts[category.id] : '—' }}</span>
          </button>
        </nav>

        <div class="sidebar-bottom">
          <button v-if="session.loggedIn" type="button" data-testid="open-publications" @click="publicationsOpen = true"><AppIcon name="file" /><span>我的发布</span><span class="sidebar-bottom-action">›</span></button>
          <button type="button" data-testid="open-settings" @click="settingsOpen = true">
            <AppIcon name="settings" /><span>{{ t("settings") }}</span><span class="sidebar-bottom-action">›</span>
          </button>
          <div v-if="sessionRestoreError" class="session-restore-error" role="status"><span>{{ sessionRestoreError }}</span><button type="button" data-testid="retry-session" :disabled="sessionRestoring" @click="restoreSavedSession">重试</button></div>
          <div class="sidebar-account">
            <button type="button" class="account-button" data-testid="open-login"
              :disabled="sessionRestoring" :title="session.loggedIn ? session.email : t('login')" @click="openAccount">
              <span class="avatar">{{ session.loggedIn ? (session.email?.[0] || "已") : "游" }}</span>
              <span>{{ sessionRestoring ? "恢复登录中…" : session.loggedIn ? session.email || t("loggedIn") : t("login") }}</span>
            </button>
            <button type="button" class="preference-toggle" :title="dark ? t('switchLight') : t('switchDark')" @click="toggleTheme">
              <AppIcon :name="dark ? 'sun' : 'moon'" />
            </button>
            <button v-if="['available','downloading','verifying','ready','installing'].includes(updateState.phase)" type="button" class="preference-toggle update-toggle" data-testid="sidebar-update" :title="updateState.phase === 'ready' ? '更新已就绪，点击安装' : '有新版本，查看更新'" aria-label="查看应用更新" @click="settingsPage = 'updates'; settingsOpen = true">
              <AppIcon name="download" /><span class="update-dot" />
            </button>
          </div>
        </div>
      </aside>

      <div v-show="!sidebarCollapsed" class="sidebar-resizer" role="separator" tabindex="0"
        aria-label="调整侧栏宽度" aria-orientation="vertical" aria-controls="workbench-sidebar"
        :aria-valuenow="sidebarWidth" :aria-valuemin="200" :aria-valuemax="sidebarMaxWidth"
        @pointerdown="startSidebarResize" @pointermove="moveSidebarResize"
        @pointerup="endSidebarResize" @pointercancel="endSidebarResize"
        @lostpointercapture="endSidebarResize" @keydown="resizeSidebarByKey" />

      <main ref="contentScroller" v-show="!hasTaskPage && contentKind === 'prompts'" data-region="content" class="content-area">
        <section class="content-header">
          <div class="content-heading">
            <SiteNotice v-if="space === 'square' && remoteCatalog?.site" :site="remoteCatalog.site" heading />
            <h1 v-else data-testid="results-heading">{{ resultsHeading }}</h1>
            <div v-if="!(space === 'square' && (squareOffline || squareBlocked))" class="browse-summary">
              <span v-if="space === 'square' && remoteCatalog?.site" data-testid="results-heading">{{ resultsHeading }}</span>
              <span class="result-count" role="status">{{ space === 'square' && squareLoading ? '正在加载…' : `共 ${space === 'square' ? squareTotal : displayedItems.length} 个结果` }}</span>
            </div>
          </div>
          <div class="content-actions" :inert="batchBusy ? '' : undefined">
            <button v-if="space === 'local' && prompts.length" type="button" class="button ghost-button" data-testid="select-prompts" @click="selecting = !selecting; selectedPrompts = []">{{ selecting ? '取消多选' : '批量整理' }}</button>
            <button v-if="space === 'square'" type="button" class="button ghost-button" :disabled="squareLoading" @click="loadSquare(true)"><AppIcon name="refresh" />{{ t("refresh") }}</button>
            <button
              v-if="space === 'square'"
              type="button"
              class="button primary-button"
              data-testid="publish-prompt"
              :disabled="remoteCatalog?.site?.publishing_open === false"
              @click="startPublish"
            >
              <AppIcon name="plus" /><span>{{ t("publish") }}</span>
            </button>
            <button
              v-else
              type="button"
              class="button primary-button"
              @click="creating = true"
            >
              <AppIcon name="plus" /><span>{{ t("create") }}</span>
            </button>
          </div>
        </section>

        <section class="filter-bar" :inert="batchBusy ? '' : undefined">
          <label class="inline-search">
            <AppIcon name="search" />
            <input
              ref="searchInput"
              v-model="query"
              type="search"
              :aria-label="space === 'square' ? '搜索标题、标签或作者' : '搜索标题或正文'"
              :placeholder="space === 'square' ? '搜索标题、标签或作者' : '搜索标题或正文'"
              @input="scheduleSearch" @compositionstart="beginSearchComposition" @compositionend="finishSearchComposition"
            >
            <kbd>{{ searchShortcutLabel }}</kbd>
          </label>
          <div class="filter-tabs" role="tablist">
            <button
              v-for="tab in filterTabs"
              :key="tab.id"
              type="button"
              :data-sort="tab.id"
              role="tab"
              :aria-selected="sortTab === tab.id"
              :class="{ active: sortTab === tab.id }"
              @click="setSort(tab.id)"
            >
              {{ tab.label }} <small v-if="space === 'local'">{{ tabCount(tab.id) }}</small>
            </button>
          </div>
          <div class="filter-spacer"></div>
          <div class="filter-controls">
            <SearchableSelect class="toolbar-select language-filter" v-if="space === 'square'" v-model="contentLanguage" aria-label="广场内容语言" :options="[{value:'zh',label:'中文优先'},{value:'original',label:'作者原文'}]" @change="onModelFilter" />
          <SearchableSelect class="toolbar-select model-filter" :aria-label="t('model')" :title="modelFilter || t('allModels')" data-testid="model-filter" v-model="modelFilter" @change="onModelFilter" :options="[{value:'',label:t('allModels')}, ...modelOptions.map(name => ({value:name,label:space === 'square' ? remoteCatalog?.models.find(item => item.id === name)?.name || name : name}))]" />
          <div class="view-switch" aria-label="视图切换">
            <button type="button" :class="{ active: view === 'grid' }" :aria-pressed="view === 'grid'" title="网格视图" @click="view = 'grid'"><AppIcon name="grid" /></button>
            <button type="button" :class="{ active: view === 'list' }" :aria-pressed="view === 'list'" title="列表视图" @click="view = 'list'"><AppIcon name="list" /></button>
          </div>
          </div>
        </section>
        <p v-if="space === 'square' && sortTab === '热门'" class="square-sort-note" data-testid="square-sort-note">按已记录下载量从高到低排序 · 仅包含开启匿名统计后的下载</p>

        <div
          v-if="space === 'square' && squareOffline"
          data-testid="square-offline"
          class="offline-banner"
        >
          <span>◌</span>
          <div>
            <strong>暂时无法连接广场</strong>
            <small>广场列表暂时不可用，本地库仍可离线使用。</small>
          </div>
          <button type="button" data-testid="retry-square" @click="loadSquare(true)">重试</button>
          <button type="button" data-testid="go-local" @click="openLocal">前往本地</button>
        </div>
        <div
          v-if="space === 'square' && squareBlocked"
          data-testid="square-blocked"
          class="offline-banner"
        >
          <span>◌</span>
          <div>
            <strong>已关闭广场访问</strong>
            <small>未请求广场接口。启动器仍只搜本地。</small>
          </div>
          <button type="button" data-testid="go-local" @click="openLocal">前往本地</button>
        </div>

        <p v-if="operationNote" role="status" data-testid="operation-note" class="use-hint">{{ operationNote }}</p>
        <p v-if="space === 'square' && catalogError && !squareOffline" role="status" class="use-hint">{{ catalogError }}；当前保留上次可用分类，点击刷新重试。</p>
        <BatchOrganize v-if="selecting && space === 'local'" :prompts="selectedRows" :groups="categoryGroups" :collections="allLocalItems.filter(item => item.member_count !== undefined)"
          :can-select-page="pagePromptIds.some(id => !selectedPrompts.includes(id))"
          @select-page="selectPage" @clear-selection="clearSelection" @busy="batchBusy = $event" :complete="finishBatch" />
        <section class="prompt-section" :aria-busy="space === 'square' && squareLoading">
          <div v-if="hasContentFilter" class="active-filters" aria-label="当前筛选" :inert="batchBusy ? '' : undefined">
            <button v-if="query.trim()" type="button" :title="query" @click="clearFilters('query')">搜索：{{ query }} <span aria-hidden="true">×</span></button>
            <button v-if="selectedId" type="button" @click="clearFilters('category')">{{ selectedLabel }} <span aria-hidden="true">×</span></button>
            <button v-if="modelFilter" type="button" @click="clearFilters('model')">{{ activeModelLabel }} <span aria-hidden="true">×</span></button>
            <button type="button" class="clear-filters" data-testid="clear-filters" @click="clearFilters()">清除筛选</button>
          </div>
          <template v-if="!(space === 'square' && (squareOffline || squareBlocked))">
          <div v-if="space === 'square' && squareLoading" class="browse-loading" role="status" data-testid="browse-loading">
            <span>正在加载提示词…</span><div v-for="n in 3" :key="n" class="loading-row" aria-hidden="true"><i></i><i></i><i></i></div>
          </div>
          <WindowedPromptGrid
            v-else-if="displayedItems.length"
            :items="space === 'square' ? displayedItems : pagedItems"
            :enabled="space === 'square'"
            :list="view === 'list'"
            :scroll-root="contentScroller"
            @near-end="loadMoreSquare()"
            data-testid="library-view"
            :data-layout="view"
          >
            <template #default="{ item, cardHeight }">
            <article
              :style="cardHeight ? { height: `${cardHeight}px` } : undefined"
              class="prompt-card"
              :class="{ collection: item.kind === 'collection', 'as-row': view === 'list', 'is-selected': selecting && selectedPrompts.includes(item.id) }"
              @click="selecting && space === 'local' && item.kind === 'prompt' ? selectPrompt(item.id) : openItem(item)"
              :inert="batchBusy ? '' : undefined"
            >
              <img decoding="async" v-if="space === 'square' && referenceImages(item).length && !failedReferenceImages[item.id]" class="square-reference-cover" :src="referenceImages(item)[0]" :alt="item.title" loading="lazy" referrerpolicy="no-referrer" @error="failedReferenceImages[item.id] = true">
              <PublishedImage v-if="space === 'square' && item.preview_asset && !referenceImages(item).length" class="square-reference-cover" :item-id="item.id" :file="item.preview_asset" :title="item.title" />
              <LocalPromptCover v-if="space === 'local' && item.kind === 'prompt' && item.image_count" :prompt-id="item.id" :title="item.title" :revision="item.updated_at" />
              <div
                v-if="item.kind === 'collection' && coverPreview(item).length"
                class="collection-card-preview"
                data-testid="collection-cover-preview"
              >
                <img decoding="async" v-for="(src, index) in coverPreview(item)" :key="index" :src="src" alt="">
              </div>
              <label v-if="selecting && space === 'local' && item.kind === 'prompt'" class="card-selection" @click.stop>
                <input type="checkbox" :checked="selectedPrompts.includes(item.id)" :aria-label="`选择 ${item.title}`" :data-select-prompt="item.id" @change="selectPrompt(item.id)" />选择
              </label>
              <div class="card-top">
                <span class="type-badge"><AppIcon :name="item.kind === 'collection' ? 'folder' : 'file'" />{{ item.kind === "collection" ? "合集" : space === "square" ? "广场" : item.source === 'downloaded' ? '已下载' : '提示词' }}</span>
                <span v-if="cardCategory(item)" class="card-category" :title="cardCategory(item)">{{ cardCategory(item) }}</span>
                <span v-if="showModelTags && item.model" class="model-tag" data-testid="model-tag">{{ item.model }}</span>
              </div>
              <h3><button type="button" class="prompt-title" @click.stop="openItem(item)">{{ item.title }}</button></h3>
              <p v-if="item.author" class="prompt-author" data-testid="prompt-author">{{ item.author }}</p>
              <p class="prompt-excerpt">
                {{ cardExcerpt(item) }}
              </p>
              <div v-if="item.asset_count" class="card-assets" data-testid="card-assets">
                <span v-if="item.image_count"><AppIcon name="image" />{{ item.image_count }} 张图片</span>
                <span v-if="item.asset_count > (item.image_count || 0)"><AppIcon name="file" />{{ item.asset_count - (item.image_count || 0) }} 个文件</span>
              </div>
              <div v-if="space === 'square'" class="square-card-metrics" data-testid="square-card-metrics">
                <span title="已记录匿名下载次数；仅统计开启上报后的成功下载"><AppIcon name="download" />{{ formatMetric(item.download_count) }} <span>下载</span></span>
                <span title="当前收藏此作品的账号数">☆ {{ formatMetric(item.favorite_count) }} <span>收藏</span></span>
              </div>
              <div class="card-footer">
                <template v-if="space === 'square'">
                  <button
                    type="button"
                    class="card-action card-primary card-icon-action"
                    :title="downloadActionLabel(item.id)"
                    :aria-label="downloadActionLabel(item.id)"
                    data-testid="download-square"
                    :disabled="downloadBusy.includes(item.id)"
                    @click.stop="downloadSquare(item)"
                  >
                    <AppIcon :name="downloadBusy.includes(item.id) ? 'refresh' : downloadedIds.includes(item.id) ? 'folder' : 'download'" /><span class="sr-only">{{ downloadActionLabel(item.id) }}</span>
                  </button>
                  <button
                    type="button"
                    class="card-action card-icon-action"
                    :title="favoriteIds.includes(item.id) ? '取消收藏' : '收藏'"
                    :aria-label="favoriteIds.includes(item.id) ? '取消收藏' : '收藏'"
                    :aria-pressed="favoriteIds.includes(item.id)"
                    data-testid="favorite-square"
                    :disabled="favoriteBusy.includes(item.id)"
                    @click.stop="favoriteSquare(item)"
                  >
                    <AppIcon name="star" /><span class="sr-only">{{ favoriteIds.includes(item.id) ? "已收藏" : "收藏" }}</span>
                  </button>
                </template>
                <template v-else-if="item.kind === 'prompt'">
                  <button type="button" class="card-action card-primary" @click.stop="startUse(item)">使用</button>
                  <button v-if="!extractVariables(item.content).length" type="button" class="card-action" :disabled="useBusy" @click.stop="quickCopy(item)">复制</button>
                </template>
                <button v-else type="button" class="card-action card-primary" @click.stop="openItem(item)">打开合集</button>
                <button type="button" v-if="contextActions(item).length" class="card-action card-more" data-testid="card-more" :aria-label="`${item.title}的更多操作`" aria-haspopup="menu" @click.stop="openContextMenu($event, item)">···</button>
              </div>
            </article>
            </template>
          </WindowedPromptGrid>
          <div v-else class="empty-state">
            <span class="empty-glyph"><AppIcon :name="space === 'square' ? 'square' : 'library'" /></span>
            <h3>{{ emptyHeading }}</h3>
            <p>{{ emptyCopy }}</p>
            <button v-if="hasContentFilter" type="button" class="button" @click="clearFilters()">清除筛选</button>
            <div v-else-if="space === 'local' && sortTab === '全部'" class="empty-actions">
              <button type="button" class="button primary-button" @click="creating = true">新建提示词</button>
              <button type="button" class="button" @click="settingsPage = 'data'; settingsOpen = true">导入文件</button>
              <button type="button" class="button" @click="openSquare">去广场挑选</button>
            </div>
          </div>
          <div v-if="space === 'square' && !squareLoading && squareItems.length" class="browse-pagination" role="status">
            <span v-if="squareMoreLoading">正在加载更多…</span>
            <template v-else-if="squareNextOffset !== null">
              <span v-if="squareMoreError">加载失败，已有内容仍可浏览。</span>
              <button type="button" class="button" data-testid="square-load-more" @click="loadMoreSquare(true)">{{ squareMoreError ? '重试加载' : '加载更多' }}</button>
            </template>
            <span v-else>已显示全部 {{ squareTotal }} 条</span>
          </div>
          <nav v-if="space === 'local' && pageCount > 1" class="browse-pagination" aria-label="提示词分页">
            <button type="button" class="button" :disabled="batchBusy || browsePage === 1" @click="changePage(browsePage - 1)">上一页</button>
            <span role="status">第 {{ browsePage }} / {{ pageCount }} 页 · 每页 48 条</span>
            <button type="button" class="button" :disabled="batchBusy || browsePage === pageCount" @click="changePage(browsePage + 1)">下一页</button>
          </nav>
          </template>
        </section>
      </main>
      <SkillsPage v-if="skillsVisited" v-show="contentKind === 'skills' && !hasTaskPage" ref="skillsPage" :mode="skillsMode" :category="skillsCategories[skillsMode]" @categories="skillsCategorySummary[$event.mode] = $event" @busy="skillsBusy = $event"/>
      <div v-show="hasTaskPage" class="task-host" data-testid="task-host">
    <section v-if="addingCategory" v-page-focus="closeCategoryDialog" class="workspace-page category-page" role="region" aria-labelledby="category-page-title">
      <header class="modal-header"><h2 id="category-page-title">新建分类</h2><button type="button" class="page-back" aria-label="返回" :disabled="categoryBusy" @click="closeCategoryDialog">← 返回</button></header>
      <div class="create-body">
            <label class="field"><span>所属分类</span>
              <SearchableSelect v-model="addingCategoryId" data-testid="category-parent" :disabled="categoryBusy" :options="[{value:'',label:'无（新建大分类）'}, ...categoryGroups.map(group => ({value:group.id,label:group.name}))]" />
            </label>
            <label class="field"><span>分类名称</span>
              <input v-model="newCategoryName" data-testid="new-category-name" :placeholder="addingCategoryId ? '小分类名称' : '大分类名称'" :disabled="categoryBusy"
                @keydown.enter.prevent="!$event.isComposing && !$event.repeat && confirmAddCategory()">
            </label>

        <p v-if="categoryError" role="alert" data-testid="category-error">{{ categoryError }}</p>
      </div>
      <footer class="modal-footer"><button type="button" class="button ghost-button" :disabled="categoryBusy" @click="closeCategoryDialog">取消</button><button type="button" class="button primary-button" data-testid="confirm-category" :disabled="categoryBusy || !newCategoryName.trim()" @click="confirmAddCategory">{{ categoryBusy ? '正在创建…' : '创建分类' }}</button></footer>
    </section>
    <CreatePromptModal
      v-if="creating || editing"
      ref="editorPage"
      v-show="!loginReason"
      :prompt="editing"
      :groups="categoryGroups"
      :model-options="modelOptions"
      :default-model="defaultModel"
      :default-category-id="selectedId === '__uncategorized__' ? '' : (selectedId || '')"
      :error="editorError"
      :busy="editorBusy"
      @cancel="closeEditor"
      @stay="pendingNavigation = null"
      @save="savePrompt"
      @remove="removePrompt"
    />
    <MyPublications v-if="publicationsOpen" v-show="!loginReason" :session="session" @cancel="publicationsOpen = false" @login="openLogin('查看我的发布')" />
    <LocalPromptDetail v-if="reading" :key="reading.id" v-show="!editing && !using && !loginReason"
      :prompt="reading" @cancel="reading = null" @edit="editing = reading" @use="startUse(reading)" />
    <UsePromptModal
      v-if="using"
      v-show="!loginReason"
      :prompt="using"
      :hints-enabled="variableHints"
      :error="useError"
      :busy="useBusy"
      @cancel="using = null"
      @copied="finishUse"
    />
    <CollectionDetailModal
      v-if="openedCollection"
      :key="openedCollection.id"
      v-show="!reading && !editing && !creating && !using && !loginReason"
      :collection="openedCollection"
      :members="collectionMembers"
      :prompts="collectionCandidates"
      :error="collectionError"
      :busy="collectionBusy"
      :loading="collectionLoading"
      :ready="collectionReady"
      @retry="openCollection(openedCollection)"
      @cancel="openedCollection = null"
      @add="addToOpenedCollection"
      @remove-member="removeFromOpenedCollection"
      @open="openCollectionMember"
      @use="useCollectionMember"
      @edit="editOpenedCollection"
    />
    <SquareDetailModal
      v-if="squareDetail"
      v-show="!loginReason"
      :item="squareDetail"
      :back-label="space === 'local' ? '返回本地列表' : '返回广场'"
      :loading="squareDetailLoading"
      :error="squareDetailError"
      :note="operationNote"
      :downloading="downloadBusy.includes(squareDetail.id)"
      :download-progress="downloadProgress[squareDetail.id] || ''"
      :downloaded="downloadedIds.includes(squareDetail.id)"
      :favorite="favoriteIds.includes(squareDetail.id)"
      :favorite-busy="favoriteBusy.includes(squareDetail.id)"
      @cancel="closeSquareDetail"
      @retry="openSquareDetail(squareDetail)"
      @download="downloadSquare(squareDetail)"
      @complete-images="completeImages(squareDetail)"
      @favorite="favoriteSquare(squareDetail)"
    />
    <LoginModal
      v-if="loginReason"
      ref="loginPage"
      :reason="loginReason"
      @cancel="closeLoginPage"
      @success="finishLogin"
    />
    <section v-if="publishResume" v-show="!loginReason" v-page-focus="() => !publishBusy && (publishResume = false)" class="workspace-page" data-testid="publish-resume" role="region" aria-labelledby="publish-title">
        <header class="modal-header">
          <div>
            <h2 id="publish-title">发布到广场</h2>
          </div>
          <button type="button" class="page-back" aria-label="返回" :disabled="publishBusy" @click="publishResume = false">← 返回</button>
        </header>
        <div class="create-body">
          <p v-if="publishSourcesLoading" role="status">正在读取本地内容…</p>
          <p v-else-if="publishSourcesError" role="alert">{{ publishSourcesError }} <button type="button" data-testid="retry-publish-sources" @click="openPublish">重新读取</button></p>
          <p v-else-if="!publishSources.length" role="status">本地库还没有内容，请返回本地提示词新建后再发布。</p>
          <label class="field">
            <span>本地内容</span>
            <SearchableSelect v-model="publishSourceId" data-testid="publish-source" :disabled="publishBusy || publishSourcesLoading || Boolean(publishSourcesError)" :options="[{value:'',label:'选择要发布的本地提示词或合集'}, ...publishSources.map(item => ({value:item.id,label:item.title}))]" />
          </label>
          <div class="publish-explainer"><AppIcon name="globe" /><div><strong>分享前确认内容可以公开</strong><p>请移除密钥、个人信息和其他不适合公开的内容。</p></div></div>
          <label v-if="remoteCatalog" class="field"><span>广场分类</span><SearchableSelect v-model="publishCategoryId" data-testid="publish-category" :disabled="publishBusy" :options="[{value:null,label:'未分类'}, ...remoteCatalog.categories.map(category => ({value:category.id,label:remoteCategoryLabel(category)}))]" /></label>
          <label v-if="remoteCatalog" class="field"><span>适用模型</span><SearchableSelect v-model="publishModel" data-testid="publish-model" :disabled="publishBusy" :options="[{value:null,label:'通用模型'}, ...(publishModel &amp;&amp; !remoteCatalog.models.some(item => item.id === publishModel) ? [{value:publishModel,label:publishModel+'（原有自定义模型）'}] : []), ...remoteCatalog.models.map(model => ({value:model.id,label:model.name}))]" /></label>
          <p v-if="catalogError" class="use-hint" role="status">{{ catalogError }}；恢复连接后请重新进入发布页更新分类。</p>
          <p v-if="operationNote" role="status" class="use-hint">{{ operationNote }}</p>
          <p>提交后本地正文仍可编辑，审核状态不会覆盖本机内容。</p>
          <fieldset class="publication-files" :disabled="publishBusy || publishAssetsLoading">
            <legend>公开附件 · 可选</legend>
            <p class="use-hint">默认不公开任何附件。勾选的文件会上传并交由人工审核，通过后所有可访问广场的人都能下载；已下载副本无法撤回。</p>
            <p v-if="publishAssetsLoading" role="status">正在读取附件…</p>
            <p v-else-if="publishAssetsError" role="alert">{{ publishAssetsError }} <button type="button" @click="loadPublishAssets">重试</button></p>
            <p v-else-if="!publishAssets.length" class="use-hint">所选内容没有附件。</p>
            <div v-if="publishAssets.some(asset => asset.mime.startsWith('image/'))" class="publication-image-summary">
              <span>已选择 {{ publishAssets.filter(asset => asset.mime.startsWith('image/') && publishAssetIds.includes(asset.id)).length }} 张图片</span>
              <button type="button" class="button ghost-button" data-testid="select-publish-images" @click="publishAssetIds = [...new Set([...publishAssetIds, ...publishAssets.filter(asset => asset.mime.startsWith('image/')).map(asset => asset.id)])]">选择全部图片</button>
              <p v-if="!publishAssets.some(asset => asset.mime.startsWith('image/') && publishAssetIds.includes(asset.id))" role="status">尚未选择图片，本次发布不会包含本地图片。</p>
            </div>
            <label v-for="asset in publishAssets" :key="asset.id" class="publication-file"><input v-model="publishAssetIds" type="checkbox" :value="asset.id" data-testid="publish-asset"><img v-if="asset.mime.startsWith('image/')" class="publication-image-preview" :src="assetUrl(asset)" :alt="asset.name" loading="lazy"><span>{{ asset.name }}<small v-if="asset.memberTitle">所属提示词：{{ asset.memberTitle }}</small><small>{{ formatBytes(assetSize(asset)) }} · {{ asset.mime }}</small></span></label>
          </fieldset>
        </div>
        <footer class="modal-footer">
          <button type="button" class="button ghost-button" :disabled="publishBusy" @click="publishResume = false">返回</button>
          <button
            type="button"
            class="button primary-button"
            data-testid="publish-submit"
            :disabled="!publishSourceId || publishBusy || publishSourcesLoading || Boolean(publishSourcesError) || publishAssetsLoading || Boolean(publishAssetsError)"
            @click="submitPublish"
          >
            {{ publishBusy ? '正在提交…' : '提交审核' }}
          </button>
        </footer>
      </section>

      </div>
    </div>

    <div v-if="deletingCategory" class="modal-layer">
      <div class="modal-backdrop" @click="closeCategoryDialog"></div>
      <section v-dialog-focus="closeCategoryDialog" class="modal category-modal" :role="deletingCategory ? 'alertdialog' : 'dialog'" aria-modal="true" aria-labelledby="category-dialog-title" :aria-busy="categoryBusy">
        <header class="modal-header">
          <h2 id="category-dialog-title">{{ deletingCategory ? '删除分类' : '新建分类' }}</h2>
          <button type="button" class="modal-close" aria-label="关闭分类窗口" :disabled="categoryBusy" @click="closeCategoryDialog">×</button>
        </header>
        <div class="create-body">
          <p v-if="deletingCategory">删除「{{ deletingCategory.name }}」？该分类中的提示词和合集会移到“未分类”，不会删除正文或合集成员。</p>
          <p v-if="categoryError" role="alert" data-testid="category-error">{{ categoryError }}</p>
        </div>
        <footer class="modal-footer">
          <button type="button" class="button ghost-button" :data-dialog-autofocus="deletingCategory ? '' : undefined" :disabled="categoryBusy" @click="closeCategoryDialog">取消</button>
          <button v-if="deletingCategory" type="button" class="button danger-button" data-testid="confirm-delete-category" :disabled="categoryBusy" @click="confirmDeleteCategory">{{ categoryBusy ? '正在删除…' : '删除分类' }}</button>
        </footer>
      </section>
    </div>

    <div v-if="pendingDelete" class="download-notice" data-testid="delete-confirmation" role="group" aria-label="确认删除">
      <span>删除「{{ pendingDelete.title }}」？{{ pendingDelete.kind === 'collection' ? '合集内的提示词会保留。' : '将从本地列表移除。' }}</span>
      <button ref="cancelDeleteButton" type="button" :disabled="editorBusy" @click="pendingDelete = null">取消</button>
      <button type="button" :disabled="editorBusy" data-testid="confirm-delete" @click="confirmRemovePrompt">{{ editorBusy ? '正在删除…' : '确认删除' }}</button>
    </div>
    <div v-if="operationNotice" class="download-notice" :class="{ 'notice-above-confirmation': pendingDelete }" :data-testid="operationNotice.kind + '-notice'" role="status" aria-live="polite">
      <span>{{ operationNotice.text }}</span>
      <button v-if="operationNotice.retry" type="button" data-testid="retry-operation-refresh" :disabled="operationRefreshBusy || space !== 'local'" @click="retryOperationRefresh">{{ space !== 'local' ? '请回到本地刷新' : operationRefreshBusy ? '正在刷新…' : '重试刷新' }}</button>
      <button v-if="operationNotice.success && operationNotice.kind === 'download'" type="button" @click="closeSquareDetail(); openLocal(); operationNotice = null">前往本地</button>
      <button type="button" aria-label="关闭提示" @click="operationNotice = null">×</button>
    </div>

    <GlobalSearch v-if="globalSearchOpen" @close="globalSearchOpen = false" @select="openGlobalResult" />

    <SettingsModal
      :inert="globalSearchOpen ? '' : undefined"
      ref="settingsView"
      v-if="settingsOpen"
      v-show="!loginReason"
      :theme="theme"
      :host="host"
      :session="session"
      :initial-page="settingsPage"
      :logout-busy="logoutBusy"
      :logout-error="logoutError"
      :language="uiLanguage"
      @cancel="closeSettings"
      @stay="pendingNavigation = null"
      @language="applyUiLanguage"
      @theme="applyTheme($event, false)"
      @publications="openPublicationsFromSettings"
      @imported="refreshLocalSettings"
      @history-cleared="reloadPrompts"
      @launcher-shortcut-saved="launcherShortcut = $event"
      @login="openLogin('登录账号')"
      @logout="logoutFromSettings"
    />

    <div
      v-if="contextMenu"
      class="context-menu-layer"
      data-testid="context-menu-layer"
      @keydown.esc.stop="closeContextMenu"
      @click="closeContextMenu"
    >
      <div
        class="context-menu"
        data-testid="context-menu"
        role="menu"
        aria-label="提示词操作"
        @keydown="onMenuKeydown"
        :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
        @click.stop
      >
        <button
          role="menuitem"
          v-for="action in contextActions(contextMenu.item)"
          :key="action.id"
          type="button"
          :data-action="action.id"
          :disabled="action.disabled"
          @click="runContextAction(action.id)"
        >
          {{ action.label }}
        </button>
      </div>
    </div>
    <footer v-show="!settingsOpen || Boolean(loginReason)" data-region="statusbar" class="statusbar" :inert="globalSearchOpen ? '' : undefined">
      <span class="status-item">
        <span class="connection-dot" :class="databaseStatus === 'ready' ? 'online' : 'offline'"></span>
        <span>{{ t("localFirst") }}</span>
      </span>
      <span class="status-sep"></span>
      <span class="status-item">{{ databaseLabel }}</span>
      <span class="status-item">本地 <strong>{{ localCount }}</strong> 条</span>
      <span class="status-spacer"></span>
      <button type="button" class="status-button" @click="$emit('open-launcher')">
        <AppIcon name="search" /> {{ t("launcher") }} <kbd>{{ shortcutLabel }}</kbd>
      </button>
    </footer>
    <NicknameSetup v-if="session.loggedIn && !profileReady" :key="session.email" @ready="finishProfile" @logout="cancelProfile" />
  </div>
</template>

<script setup>
import SearchableSelect from "./SearchableSelect.vue";
import { formatMetric } from '../platform/contentMetrics.js';
import AppIcon from "./AppIcon.vue";
import GlobalSearch from "./GlobalSearch.vue";
import { listPromptAssets, assetSize, formatBytes, assetUrl } from '../platform/assets.js';
import { uploadPrivateAsset } from '../platform/privateMedia.js';
import { vDialogFocus } from "../lib/dialogFocus.js";
import { vPageFocus } from "../lib/pageFocus.js";
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import CollectionDetailModal from "./CollectionDetailModal.vue";
import CreatePromptModal from "./CreatePromptModal.vue";
import LoginModal from "./LoginModal.vue";
import NicknameSetup from "./NicknameSetup.vue";
import SettingsModal from "./SettingsModal.vue";
import { updateState, startUpdateChecks } from "../platform/updates.js";
let stopUpdateChecks;
onMounted(() => { stopUpdateChecks = startUpdateChecks(); });
onUnmounted(() => stopUpdateChecks?.());
import SquareDetailModal from "./SquareDetailModal.vue";
import MyPublications from './MyPublications.vue';
import BatchOrganize from './BatchOrganize.vue';
import LocalPromptDetail from './LocalPromptDetail.vue';
import { extractVariables } from '../lib/renderPrompt.js';
import UsePromptModal from "./UsePromptModal.vue";
import { getSession, logoutSession, restoreSession } from "../platform/session.js";
import { filterLocalItems, listLocalFavoriteIds, toggleLocalFavorite } from "../platform/localFavorites.js";
import { parseModelNames } from "../platform/modelCatalog.js";
import { uiText } from "../platform/uiStrings.js";
import { downloadSquareItem, completeSquareImages, fetchSquareContent, fetchSquareCatalog, listSquarePage } from "../platform/square.js";
import WindowedPromptGrid from './WindowedPromptGrid.vue';
import {skillCategories} from '../platform/skillCategories.js';
import SkillsPage from './SkillsPage.vue';
import SiteNotice from '../../../shared/SiteNotice.vue';
import { applyQueuedFavorites, favoriteWithQueue, publishWithQueue } from "../platform/syncQueue.js";
import { parseCoverUrls } from "../lib/cover.js";
import PublishedImage from "./PublishedImage.vue";
import { referenceImages } from "../lib/squareReference.js";
import LocalPromptCover from './LocalPromptCover.vue';
const failedReferenceImages = ref({});
import { DEFAULT_LAUNCHER_SHORTCUT } from "../platform/shortcut.js";
import { applyHostChrome, detectHost, formatShortcutLabel, trafficLightInsetPx } from "../platform/windowChrome.js";
import {
  addPromptToCollection,
  removePromptFromCollection,
  updateLocalCollection,
  deleteLocalCollection,
  buildCategoryTree,
  createLocalCategory,
  deleteLocalCategory,
  createLocalCollection,
  createLocalPrompt,
  deleteLocalPrompt,
  getLocalSetting,
  listCollectionMembers,
  listLocalCategories,
  listLocalCollections,
  listLocalPrompts,
  recordLocalPromptUse,
  setLocalSetting,
  updateLocalPrompt,
} from "../platform/library.js";
import "../styles/workbench-chrome.css";

const props = defineProps({
  databaseStatus: { type: String, default: "pending" },
  localCount: { type: Number, default: 0 },
  host: { type: String, default: () => detectHost() },
});
const trafficInset = computed(() => trafficLightInsetPx(props.host));
const launcherShortcut = ref(DEFAULT_LAUNCHER_SHORTCUT);
const shortcutLabel = computed(() => formatShortcutLabel(launcherShortcut.value, props.host));
const searchShortcutLabel = computed(() => formatShortcutLabel(props.host === 'macos' ? 'Super+F' : 'Control+F', props.host));
const searchInput = ref(null);
const globalSearchOpen = ref(false);
const globalSearchShortcutLabel = computed(() => formatShortcutLabel(props.host === 'macos' ? 'Super+K' : 'Control+K', props.host));
const globalSearchBlocked = computed(() => skillsBusy.value || batchBusy.value || editorBusy.value || useBusy.value || publishBusy.value || categoryBusy.value || collectionBusy.value || loginPage.value?.busy || settingsView.value?.busy || downloadBusy.value.length > 0 || favoriteBusy.value.length > 0 || Boolean(pendingDelete.value || deletingCategory.value || pendingNavigation.value));
function openGlobalSearch() {
  if (globalSearchBlocked.value || document.querySelector('[aria-modal="true"]')) return;
  closeContextMenu();
  globalSearchOpen.value = true;
}
async function openGlobalResult({ item, scope }) {
  globalSearchOpen.value = false;
  await nextTick();
  const action = () => {
    if (scope === 'square') {
      favoriteIds.value = favoriteIds.value.filter(id => id !== item.id);
      if (item.is_favorite) favoriteIds.value.push(item.id);
      openSquareDetail(item);
    }
    else if (item.kind === 'collection') openCollection(item);
    else reading.value = item;
  };
  if (settingsOpen.value && !loginReason.value) {
    pendingNavigation.value = action;
    settingsView.value?.requestClose();
  } else navigateTo(action);
}

function focusSearch() {
  searchInput.value?.focus();
  searchInput.value?.select();
}

async function loadLauncherShortcut() {
  launcherShortcut.value = (await getLocalSetting('launcher_shortcut')) || DEFAULT_LAUNCHER_SHORTCUT;
}

const emit = defineEmits(["open-launcher", "library-changed"]);

const space = ref("local");
const contentKind = ref("prompts"), skillsMode = ref("local"), skillsVisited = ref(false), skillsBusy = ref(false), skillsPage = ref(null);
const skillsCategories = ref({local:'',square:''}), skillsCategorySummary = ref({local:null,square:null});
function selectSkillCategory(id) { if (skillsBusy.value) return; if (skillsCategories.value[skillsMode.value] === id) skillsPage.value?.showList?.(); else skillsCategories.value[skillsMode.value] = id; }
function openSkills(mode) { if (skillsBusy.value) return; cancelSquare(); contentKind.value = "skills"; skillsMode.value = mode; skillsVisited.value = true; }
const sidebarCollapsed = ref(false);
const viewportWidth = ref(window.innerWidth);
const preferredSidebarWidth = ref(null);
const sidebarMaxWidth = computed(() => Math.max(200, Math.min(400, viewportWidth.value - 560)));
const sidebarWidth = computed(() => Math.max(200, Math.min(sidebarMaxWidth.value,
  preferredSidebarWidth.value ?? (viewportWidth.value <= 760 ? 200 : 260))));
const sidebarDrag = ref(null);

function startSidebarResize(event) {
  if (event.button !== 0 || sidebarDrag.value) return;
  event.preventDefault();
  event.currentTarget.setPointerCapture(event.pointerId);
  sidebarDrag.value = { id: event.pointerId, x: event.clientX, width: sidebarWidth.value, target: event.currentTarget };
}
function moveSidebarResize(event) {
  const drag = sidebarDrag.value;
  if (!drag || event.pointerId !== drag.id) return;
  preferredSidebarWidth.value = Math.max(200, Math.min(sidebarMaxWidth.value, drag.width + event.clientX - drag.x));
}
function endSidebarResize(event) {
  const drag = sidebarDrag.value;
  if (!drag || (event?.pointerId !== undefined && event.pointerId !== drag.id)) return;
  sidebarDrag.value = null;
  if (drag.target.hasPointerCapture(drag.id)) drag.target.releasePointerCapture(drag.id);
  saveLayout();
}
function resizeSidebarByKey(event) {
  if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
  event.preventDefault();
  preferredSidebarWidth.value = Math.max(200, Math.min(sidebarMaxWidth.value, sidebarWidth.value + (event.key === 'ArrowRight' ? 10 : -10)));
  saveLayout();
}
function updateSidebarViewport() {
  endSidebarResize();
  viewportWidth.value = window.innerWidth;
}
watch(sidebarCollapsed, () => endSidebarResize());
onMounted(() => {
  window.addEventListener('resize', updateSidebarViewport);
  window.addEventListener('blur', endSidebarResize);
});
onUnmounted(() => {
  endSidebarResize();
  window.removeEventListener('resize', updateSidebarViewport);
  window.removeEventListener('blur', endSidebarResize);
});

function handleWorkbenchShortcut(event) {
  if (session.value.loggedIn && !profileReady.value) return;
  if (batchBusy.value || skillsBusy.value) return;
  const modifier = props.host === 'macos' ? event.metaKey : event.ctrlKey;
  if (!modifier || event.altKey || event.shiftKey || event.repeat || event.isComposing || event.keyCode === 229) return;
  if (globalSearchOpen.value) return;
  if (event.key.toLowerCase() === 'k') { event.preventDefault(); openGlobalSearch(); return; }
  if (addingCategory.value || deletingCategory.value) return;
  if (publicationsOpen.value || reading.value || creating.value || editing.value || using.value || openedCollection.value || loginReason.value || pendingPublish.value || squareDetail.value) return;
  if (event.key === ',') { event.preventDefault(); settingsOpen.value = true; return; }
  if (event.key.toLowerCase() === 'f' && !settingsOpen.value && !publishResume.value) {
    event.preventDefault(); if (contentKind.value === 'skills') { skillsPage.value?.focusSearch?.(); } else { focusSearch(); } return;
  }
  const target = event.target;
  if (event.key.toLowerCase() === 'b' && !settingsOpen.value && !target?.closest?.('input, textarea, select, [contenteditable="true"]')) {
    event.preventDefault(); sidebarCollapsed.value = !sidebarCollapsed.value;
  }
}
onMounted(() => window.addEventListener('keydown', handleWorkbenchShortcut));
onUnmounted(() => window.removeEventListener('keydown', handleWorkbenchShortcut));
const selectedId = ref(null);
const dark = ref(false);
const view = ref("grid");
const sortTab = ref("全部");
const contentLanguage = ref("zh");
const creating = ref(false);
const editing = ref(null);
const reading = ref(null);
const publicationsOpen = ref(false);
const selecting = ref(false), selectedPrompts = ref([]), batchBusy = ref(false);
const selectedRows = computed(() => prompts.value.filter(p => selectedPrompts.value.includes(p.id)));
function selectPrompt(id) {
  if (batchBusy.value) return;
  selectedPrompts.value = selectedPrompts.value.includes(id) ? selectedPrompts.value.filter(value => value !== id) : [...selectedPrompts.value, id];
}
async function finishBatch(ids) {
  selectedPrompts.value = selectedPrompts.value.filter(id => !ids.includes(id));
  try { await reloadPrompts(); }
  catch (error) { notifyOperation(`整理已完成，但刷新失败：${error.message || error}`, false, 'collection', true); }
}
const using = ref(null);
const useError = ref("");
const useBusy = ref(false);
const editorError = ref("");
const editorBusy = ref(false);
const editorPage = ref(null), loginPage = ref(null);
const pendingNavigation = ref(null);
const collectionError = ref("");
const collectionBusy = ref(false);
const collectionLoading = ref(false);
const collectionReady = ref(false);
let collectionRequest = 0;
onUnmounted(() => { ++collectionRequest; });
const collectionCandidates = ref([]);
const settingsOpen = ref(false);
const settingsPage = ref('general');
const settingsView = ref(null);
const logoutBusy = ref(false), logoutError = ref('');
const openedCollection = ref(null);
const collectionMembers = ref([]);
const query = ref("");
const prompts = ref([]);
const collections = ref([]);
const allLocalItems = ref([]);
const categoryGroups = ref([]);
const categoryTree = ref(null);
const remoteCatalog = ref(null), remoteCategoryGroups = ref([]), catalogError = ref('');
const visibleCategoryGroups = computed(() => space.value === 'square' && remoteCatalog.value ? remoteCategoryGroups.value : categoryGroups.value.filter(group => space.value === 'local' || group.is_system));
const addingCategory = ref(false);
const addingCategoryId = ref("");
const deletingCategory = ref(null);
const categoryBusy = ref(false);
const newCategoryName = ref("");
const categoryError = ref("");
const theme = ref("light");
const session = ref(getSession());
const profileReady = ref(false);
watch(() => session.value.email, () => { profileReady.value = false; }, { flush: 'sync' });
const sessionRestoring = ref(false), sessionRestoreError = ref('');
async function restoreSavedSession() {
  if (sessionRestoring.value) return;
  sessionRestoring.value = true;
  sessionRestoreError.value = '';
  const noticeTimer = setTimeout(() => { sessionRestoreError.value = '恢复较慢，如有系统钥匙串提示，请先完成授权'; }, 12000);
  try {
    await restoreSession();
    sessionRestoreError.value = '';
    session.value = getSession();
  } catch {
    sessionRestoreError.value = '登录暂未恢复，请检查网络或钥匙串权限';
  } finally { clearTimeout(noticeTimer); sessionRestoring.value = false; }
  if (session.value.loggedIn) {
    await refreshFavorites();
    if (space.value === 'square') await loadSquare();
  }
}
onMounted(restoreSavedSession);
const loginReason = ref("");
const publishResume = ref(false);
const pendingPublish = ref(false);
const publishSources = ref([]);
const publishSourcesLoading = ref(false), publishSourcesError = ref('');
let publishSourcesRequest = 0;
watch(publishResume, value => { if (!value) ++publishSourcesRequest; }, { flush: 'sync' });
onUnmounted(() => { ++publishSourcesRequest; });
const publishSourceId = ref("");
const publishCategoryId = ref(null), publishModel = ref(null);
const publishAssets = ref([]), publishAssetIds = ref([]), publishAssetsLoading = ref(false), publishAssetsError = ref('');
let publishAssetsVersion = 0;
async function loadPublishAssets() {
  const version = ++publishAssetsVersion;
  publishAssets.value = []; publishAssetIds.value = []; publishAssetsError.value = '';
  const source = publishSources.value.find(item => item.id === publishSourceId.value);
  publishAssetsLoading.value = Boolean(source);
  if (!publishAssetsLoading.value) return;
  try {
    const assets = [];
    if (source.kind === 'collection') {
      for (const member of await listCollectionMembers(source.id)) {
        const files = await listPromptAssets(member.id);
        // Imported copies can share local asset IDs; each publication occurrence needs its own ID.
        assets.push(...files.map(file => ({ ...file, id: crypto.randomUUID(), memberId: member.id, memberTitle: member.title })));
      }
    } else assets.push(...await listPromptAssets(source.id));
    if (version === publishAssetsVersion) publishAssets.value = assets;
  }
  catch (error) { if (version === publishAssetsVersion) publishAssetsError.value = `附件读取失败：${error.message || error}`; }
  finally { if (version === publishAssetsVersion) publishAssetsLoading.value = false; }
}
watch(() => session.value.email, () => { publishAssetIds.value = []; });
onUnmounted(() => { ++publishAssetsVersion; });
watch(publishSourceId, id => {
  const source = publishSources.value.find(item => item.id === id);
  publishCategoryId.value = publicationCategory(source?.category_id);
  publishModel.value = source?.model || null;
  loadPublishAssets();
});
const publishBusy = ref(false);
const favoriteBusy = ref([]);
const operationNote = ref("");
let layoutReady = false, layoutDisposed = false, layoutWrites = Promise.resolve();
async function loadLayout() {
  try {
    const raw = await getLocalSetting('workbench_layout');
    const saved = raw ? JSON.parse(raw) : {};
    if (layoutDisposed) return;
    if (Number.isFinite(saved?.width) && saved.width >= 200 && saved.width <= 400) preferredSidebarWidth.value = saved.width;
    if (typeof saved?.collapsed === 'boolean') sidebarCollapsed.value = saved.collapsed;
    if (['grid', 'list'].includes(saved?.view)) view.value = saved.view;
  } catch { operationNote.value = '布局偏好读取失败，已使用默认布局。'; }
  await nextTick();
  layoutReady = !layoutDisposed;
}
function saveLayout() {
  if (!layoutReady || layoutDisposed) return;
  const value = JSON.stringify({ width: preferredSidebarWidth.value ?? 260, collapsed: sidebarCollapsed.value, view: view.value });
  layoutWrites = layoutWrites.then(() => setLocalSetting('workbench_layout', value))
    .catch(() => { operationNote.value = '布局偏好保存失败，本次调整仍可使用。'; });
}
watch([sidebarCollapsed, view], saveLayout);
onMounted(loadLayout);
onUnmounted(() => { layoutDisposed = true; });
const squareItems = ref([]);
const squareCategoryCounts = ref(null), squareCategoryTotal = ref(null);
const contentScroller = ref(null);
const squareTotal = ref(0), squareNextOffset = ref(null), squareMoreLoading = ref(false), squareMoreError = ref(false);
let squareController;
function cancelSquare() { squareController?.abort(); }
const squareOffline = ref(false);
const squareLoading = ref(false);
const squareBlocked = ref(false);
const favoriteIds = ref([]);
const localFavoriteIds = ref([]);
const contextMenu = ref(null);
const modelFilter = ref("");
const modelCatalogText = ref("");
const customModelsText = ref("");
const seenModels = ref([]);
const defaultModel = ref("");
const showModelTags = ref(true);
const variableHints = ref(false);
const uiLanguage = ref("zh");

function t(key) {
  return uiText(uiLanguage.value, key);
}
function cardExcerpt(item) {
  if (item.kind === 'collection') return `${item.member_count ?? 0} 个提示词`;
  const text = item.content || item.excerpt || '还没有正文';
  const excerpt = Array.from(text.slice(0, 482)).slice(0, 240).join('');
  return excerpt.length < text.length ? `${excerpt}…` : text;
}
const libraryItems = computed(() => [
  ...collections.value.map((item) => ({ ...item, kind: "collection" })),
  ...prompts.value.map((item) => ({ ...item, kind: "prompt" })),
]);
const displayedItems = computed(() => {
  const rows =
    space.value === "square"
      ? squareItems.value
      : filterLocalItems(libraryItems.value, {
          tab: sortTab.value,
          favoriteIds: localFavoriteIds.value,
        });
  if (!modelFilter.value) return rows;
  return rows.filter((item) => item.kind === "prompt" && item.model === modelFilter.value);
});
const browsePage = ref(1);
const pageCount = computed(() => Math.max(1, Math.ceil(displayedItems.value.length / 48)));
const pagedItems = computed(() => displayedItems.value.slice((browsePage.value - 1) * 48, browsePage.value * 48));
const pagePromptIds = computed(() => pagedItems.value.filter(item => item.kind === 'prompt').map(item => item.id));
function selectPage() {
  if (!batchBusy.value) selectedPrompts.value = [...new Set([...selectedPrompts.value, ...pagePromptIds.value])];
}
function clearSelection() {
  if (!batchBusy.value) selectedPrompts.value = [];
}
async function changePage(page) {
  if (batchBusy.value) return;
  browsePage.value = Math.max(1, Math.min(page, pageCount.value));
  await nextTick();
  if (contentScroller.value) contentScroller.value.scrollTop = 0;
}
watch([space, query, selectedId, modelFilter, sortTab], () => {
  browsePage.value = 1;
  clearSelection();
});
watch(pageCount, count => { browsePage.value = Math.min(browsePage.value, count); });
const filterTabs = computed(() =>
  space.value === "square"
    ? [
        { id: "推荐", label: t("tabRecommended") },
        { id: "最新", label: t("tabLatest") },
        { id: "热门", label: t("tabHot") },
        { id: "收藏", label: t("tabFavorite") },
      ]
    : [
        { id: "全部", label: t("tabAll") },
        { id: "最近", label: t("tabRecent") },
        { id: "收藏", label: t("tabFavorite") },
      ],
);
const selectedLabel = computed(() => {
  if (!selectedId.value) return t("allPrompts");
  if (selectedId.value === "__uncategorized__") return uiLanguage.value === "en" ? "Uncategorized" : "未分类";
  for (const group of visibleCategoryGroups.value) {
    if (group.id === selectedId.value) return group.name;
    const child = group.children.find((item) => item.id === selectedId.value);
    if (child) return child.name;
  }
  return t("allPrompts");
});

const modelOptions = computed(() =>
  space.value === 'square' && remoteCatalog.value ? remoteCatalog.value.models.map(item => item.id) : parseModelNames(modelCatalogText.value, customModelsText.value, seenModels.value, prompts.value),
);
const hasContentFilter = computed(() => Boolean(query.value.trim() || selectedId.value || modelFilter.value));
const activeModelLabel = computed(() => space.value === 'square' ? remoteCatalog.value?.models.find(item => item.id === modelFilter.value)?.name || modelFilter.value : modelFilter.value);
const resultsHeading = computed(() => {
  if (query.value.trim()) return '搜索结果';
  if (selectedId.value) return selectedLabel.value;
  if (space.value === 'local') return sortTab.value === '全部' ? selectedLabel.value : sortTab.value === '最近' ? '最近使用' : '我的收藏';
  return ({ 推荐: '推荐提示词', 最新: '最新发布', 热门: '热门提示词', 收藏: '我的收藏' })[sortTab.value];
});
async function clearFilters(field) {
  if (batchBusy.value) return;
  cancelSearch();
  if (!field || field === 'query') query.value = '';
  if (!field || field === 'category') selectedId.value = null;
  if (!field || field === 'model') modelFilter.value = '';
  if (space.value === 'square') await loadSquare(); else await reloadPrompts();
}
const emptyHeading = computed(() => {
  if (space.value === 'square' && squareOffline.value) return t('emptyOffline');
  if (hasContentFilter.value) return t('emptyFiltered');
  if (space.value === "square") return sortTab.value === '收藏' ? t('emptyFavorite') : t("emptySquare");
  if (sortTab.value === "最近") return t("emptyRecent");
  if (sortTab.value === "收藏") return t("emptyFavorite");
  return t("emptyLocal");
});
const emptyCopy = computed(() => {
  if (hasContentFilter.value && !(space.value === 'square' && squareOffline.value)) return t('emptyFilteredHint');
  if (sortTab.value === '最近') return '使用过的提示词会出现在这里，方便下次继续。';
  if (sortTab.value === '收藏') return space.value === 'local' ? '点击提示词的「···」选择收藏，在这里快速找到常用内容。' : '收藏喜欢的社区提示词后，可在这里再次找到。';
  return space.value === 'square' ? t('emptySquareHint') : t('emptyLocalHint');
});
const locationLabel = computed(() => contentKind.value === "skills" ? (skillsMode.value === "local" ? "本机 Skills" : "Skill 广场") : (space.value === "square" ? t("square") : t("local")));
const hasTaskPage = computed(() => Boolean(publicationsOpen.value || reading.value || creating.value || editing.value || using.value || openedCollection.value || squareDetail.value || loginReason.value || publishResume.value || addingCategory.value));
const taskTitle = computed(() => publicationsOpen.value ? '我的发布' : reading.value && !editing.value && !using.value ? reading.value.title : loginReason.value ? '登录账号' : creating.value ? '新建' : editing.value ? '编辑' : using.value ? '使用提示词' : openedCollection.value ? openedCollection.value.title : squareDetail.value ? squareDetail.value.title : publishResume.value ? '发布到广场' : addingCategory.value ? '新建分类' : '');

function guardSidebarNavigation(event) {
  if (batchBusy.value || skillsBusy.value) { event.preventDefault(); event.stopPropagation(); return; }
  if (!hasTaskPage.value) return;
  const button = event.target.closest('button');
  if (!button || button.matches('.preference-toggle, .tree-expand, .category-collapse')) return;
  event.preventDefault();
  event.stopPropagation();
  navigateTo(() => button.click());
}

function navigateTo(action) {
  if (batchBusy.value || skillsBusy.value) return;
  if (!hasTaskPage.value) { action(); return; }
  if (editorBusy.value || useBusy.value || publishBusy.value || categoryBusy.value || collectionBusy.value || loginPage.value?.busy || settingsView.value?.busy || downloadBusy.value.length || favoriteBusy.value.length) return;
  pendingNavigation.value = action;
  if (loginReason.value) loginPage.value?.close();
  else if (creating.value || editing.value) editorPage.value?.requestClose();
  else finishNavigation();
}

function finishNavigation() {
  const action = pendingNavigation.value;
  if (!action) return;
  if (settingsOpen.value) {
    loginReason.value = '';
    nextTick(() => settingsView.value?.requestClose());
    return;
  }
  pendingNavigation.value = null;
  pendingDelete.value = null;
  creating.value = false; editing.value = null; using.value = null; reading.value = null; publicationsOpen.value = false;
  openedCollection.value = null; closeSquareDetail();
  loginReason.value = ''; publishResume.value = false; pendingPublish.value = false;
  closeCategoryDialog();
  nextTick(action);
}

function closeLoginPage() {
  loginReason.value = '';
  pendingPublish.value = false;
  finishNavigation();
}
const databaseLabel = computed(() => {
  if (props.databaseStatus === "ready") return "SQLite 就绪";
  if (props.databaseStatus === "failed") return "SQLite 失败";
  return "SQLite 未接入";
});

function toggleGroup(group) {
  group.open = !group.open;
}

function collapseAll() {
  for (const group of visibleCategoryGroups.value) group.open = false;
}

function selectCategory(id) {
  selectedId.value = id;
  return space.value === "square" ? loadSquare() : reloadPrompts();
}

function categoryCount(id) {
  if (space.value === 'square') {
    if (!squareCategoryCounts.value) return '—';
    const parents = remoteCatalog.value?.category_parents ?? Object.fromEntries((remoteCatalog.value?.categories ?? []).map(category => [category.id, category.parent_id]));
    return Object.entries(squareCategoryCounts.value).reduce((total, [category, count]) => total + (category === id || parents[category] === id ? count : 0), 0);
  }
  return allLocalItems.value.filter((item) => {
    const category = categoryById(item.category_id);
    if (id === "__uncategorized__") return !category;
    return item.category_id === id || category?.parent_id === id;
  }).length;
}

function categoryById(id) {
  for (const group of categoryGroups.value) {
    if (group.id === id) return group;
    const child = group.children.find((item) => item.id === id);
    if (child) return child;
  }
  return null;
}

function startAddCategory() {
  if (space.value !== 'local') return;
  categoryError.value = "";
  addingCategory.value = true;
  const current = categoryById(selectedId.value);
  addingCategoryId.value = current?.parent_id || current?.id || '';
  newCategoryName.value = "";
}

async function confirmAddCategory() {
  if (categoryBusy.value || !addingCategory.value) return;
  categoryBusy.value = true;
  categoryError.value = "";
  try {
    const created = await createLocalCategory({ name: newCategoryName.value, parentId: addingCategoryId.value || null });
    await refreshCategoryTree(created.parent_id || created.id);
    await selectCategory(created.id);
    addingCategory.value = false;
    await nextTick();
    categoryTree.value?.querySelector('.tree-row.active')?.scrollIntoView?.({ block: 'nearest' });
  } catch (error) {
    categoryError.value = error instanceof Error ? error.message : String(error);
  } finally { categoryBusy.value = false; }
}

function closeCategoryDialog() {
  if (categoryBusy.value) return;
  addingCategory.value = false;
  addingCategoryId.value = '';
  deletingCategory.value = null;
  categoryError.value = '';
}

function startDeleteCategory(category) {
  if (space.value !== 'local' || category.is_system) return;
  categoryError.value = '';
  deletingCategory.value = category;
}

async function refreshCategoryTree(openParent) {
  const openIds = new Set(categoryGroups.value.filter(group => group.open).map(group => group.id));
  categoryGroups.value = buildCategoryTree(await listLocalCategories());
  for (const group of categoryGroups.value) group.open = group.id === openParent || openIds.has(group.id);
}

async function confirmDeleteCategory() {
  if (categoryBusy.value || !deletingCategory.value) return;
  categoryBusy.value = true;
  categoryError.value = '';
  let deleted = false;
  const title = deletingCategory.value.name;
  try {
    const id = deletingCategory.value.id;
    await deleteLocalCategory(id);
    deleted = true;
    deletingCategory.value = null;
    notifyOperation(`已删除分类「${title}」，内容已保留。`, true, 'delete');
    await refreshCategoryTree();
    if (selectedId.value === id) selectedId.value = '__uncategorized__';
    await reloadPrompts();
    deletingCategory.value = null;
  } catch (error) {
    categoryError.value = `${deleted ? '分类已删除，但刷新失败' : '删除失败'}：${error.message || error}`;
    notifyOperation(categoryError.value, false, 'delete');
  }
  finally { categoryBusy.value = false; }
}

function openLogin(reason) {
  loginReason.value = reason;
}

async function logoutFromSettings() {
  if (logoutBusy.value) return;
  logoutBusy.value = true;
  logoutError.value = '';
  try {
    await logoutSession();
    session.value = getSession();
    favoriteIds.value = [];
    if (space.value === 'square') { if (sortTab.value === '收藏') sortTab.value = '推荐'; await loadSquare(); }
  } catch (error) { logoutError.value = error.message || String(error); }
  finally { logoutBusy.value = false; }
}

function openAccount() {
  session.value = getSession();
  if (!session.value.loggedIn) { openLogin(t('login')); return; }
  settingsPage.value = 'account';
  settingsOpen.value = true;
}

const squareDetail = ref(null);
const squareDetailLoading = ref(false);
const squareDetailError = ref("");
const downloadBusy = ref([]);
const downloadProgress = ref({});
function downloadLabel(id) { return downloadProgress.value[id] || '下载中…'; }
function updateDownloadProgress(id, progress) {
  if (downloadsDisposed || !downloadBusy.value.includes(id)) return;
  downloadProgress.value = { ...downloadProgress.value, [id]: progress.stage === 'saving'
    ? '正在保存…' : `参考图 ${progress.completed} / ${progress.total}` };
}
function clearDownloadProgress(id) {
  const next = { ...downloadProgress.value }; delete next[id]; downloadProgress.value = next;
}
const downloadedIds = ref([]);
const operationNotice = ref(null);
const operationRefreshBusy = ref(false);
const pendingDelete = ref(null);
const cancelDeleteButton = ref(null);
let operationNoticeTimer;
onUnmounted(() => clearTimeout(operationNoticeTimer));

function notifyOperation(text, success, kind = 'download', retry = false) {
  clearTimeout(operationNoticeTimer);
  operationNotice.value = { text, success, kind, retry };
  if (!retry) operationNoticeTimer = setTimeout(() => { operationNotice.value = null; }, success ? 6000 : 12000);
}

async function refreshOperationView() {
  if (openedCollection.value) {
    const id = openedCollection.value.id;
    const request = collectionRequest;
    const updated = (await listLocalCollections({ query: '', categoryId: null })).find(item => item.id === id);
    if (updated && openedCollection.value?.id === id && request === collectionRequest) {
      if (!await openCollection(updated)) throw Error(collectionError.value);
    }
  }
  await reloadPrompts();
}

async function retryOperationRefresh() {
  if (space.value !== 'local' || operationRefreshBusy.value || !operationNotice.value?.retry) return;
  const notice = operationNotice.value;
  operationRefreshBusy.value = true;
  try {
    await refreshOperationView();
    if (operationNotice.value === notice) notifyOperation('已刷新，本次没有重复执行已完成的操作。', true, notice.kind);
  } catch (error) {
    if (operationNotice.value === notice) notifyOperation(`刷新仍失败：${error.message || error}。已完成的操作不会重复执行。`, false, notice.kind, true);
  } finally { operationRefreshBusy.value = false; }
}

async function refreshDownloaded() {
  const rows = await listLocalPrompts({ query: '', categoryId: null });
  downloadedIds.value = [...new Set(rows.map(row => row.remote_id).filter(Boolean))];
  emit('library-changed', rows.length);
}
let detailRequest = 0;

function closeSquareDetail() {
  detailRequest += 1;
  squareDetail.value = null;
}

async function openSquareDetail(item) {
  const request = ++detailRequest;
  squareDetail.value = { ...item };
  squareDetailLoading.value = true;
  squareDetailError.value = "";
  operationNote.value = "";
  try {
    const content = await fetchSquareContent(item.id);
    if (request === detailRequest) squareDetail.value = { ...squareDetail.value, ...content, download_count: squareDetail.value.download_count, favorite_count: squareDetail.value.favorite_count };
  } catch (error) {
    if (request === detailRequest) squareDetailError.value = `读取详情失败：${error.message || error}`;
  } finally {
    if (request === detailRequest) squareDetailLoading.value = false;
  }
}

const extraLauncherUnlisteners = [];
onUnmounted(() => extraLauncherUnlisteners.forEach(stop => stop()));
let downloadsDisposed = false;
onUnmounted(() => { downloadsDisposed = true; });

function downloadActionLabel(id) { return downloadBusy.value.includes(id) ? downloadLabel(id) : downloadedIds.value.includes(id) ? '打开本地副本' : '下载'; }

async function downloadSquare(item) {
  if (downloadBusy.value.includes(item.id)) return;
  if (downloadedIds.value.includes(item.id)) {
    await openDownloaded(item);
    return;
  }
  downloadBusy.value = [...downloadBusy.value, item.id];
  try {
    await downloadSquareItem(item.id, count => {
      if (downloadsDisposed) return;
      squareItems.value = squareItems.value.map(row => row.id === item.id ? { ...row, download_count: count } : row);
      if (squareDetail.value?.id === item.id) squareDetail.value = { ...squareDetail.value, download_count: count };
    }, progress => updateDownloadProgress(item.id, progress));
    downloadedIds.value = [...new Set([...downloadedIds.value, item.id])];
    operationNote.value = `「${item.title}」已下载到本地。`;
    notifyOperation(operationNote.value, true);
  } catch (error) {
    operationNote.value = `下载失败：${error.message || error}`;
    notifyOperation(operationNote.value, false);
  } finally { downloadBusy.value = downloadBusy.value.filter((id) => id !== item.id); clearDownloadProgress(item.id); }
  // Saving succeeded independently of refreshing the visible local list.
  try { await refreshDownloaded(); await reloadPrompts(); } catch { /* Retain confirmed download state. */ }
}

async function completeImages(item) {
  if(downloadBusy.value.includes(item.id)) return;
  downloadBusy.value=[...downloadBusy.value,item.id];
  try {
    await completeSquareImages(item.id, progress => updateDownloadProgress(item.id, progress));
    notifyOperation(`「${item.title}」参考图已补全，正文和已有附件保持不变。`,true);
  } catch(error) { notifyOperation(`补图失败：${error.message || error}`,false); }
  finally { downloadBusy.value=downloadBusy.value.filter(id=>id!==item.id); clearDownloadProgress(item.id); }
  try { await refreshDownloaded(); await reloadPrompts(); } catch { /* Saved images remain available after refresh. */ }
}

async function favoriteSquare(item) {
  if (favoriteBusy.value.includes(item.id)) return;
  if (!getSession().loggedIn) {
    openLogin("收藏需要登录");
    return;
  }
  const removing = favoriteIds.value.includes(item.id);
  const email = getSession().email;
  favoriteBusy.value = [...favoriteBusy.value, item.id];
  try {
    const result = await favoriteWithQueue(item.id, removing ? "DELETE" : "PUT");
    if (getSession().email !== email) return;
    favoriteIds.value = removing
      ? favoriteIds.value.filter((id) => id !== item.id)
      : [...favoriteIds.value, item.id];
    const patch = row => ({ ...row, is_favorite: !removing, favorite_count: result.queued ? row.favorite_count : Math.max(0, (Number(row.favorite_count) || 0) + (removing ? -1 : 1)) });
    squareItems.value = squareItems.value.map(row => row.id === item.id ? patch(row) : row);
    if (squareDetail.value?.id === item.id) squareDetail.value = patch(squareDetail.value);
    operationNote.value = result.queued ? "已保存到本机队列，尚未送达服务器。" : (removing ? "已取消收藏。" : "已收藏。");
    notifyOperation(operationNote.value, !result.queued);
    if (removing && sortTab.value === "收藏" && squareItems.value.some(row => row.id === item.id)) {
      // A deletion shifts subsequent SQL offsets; invalidate any in-flight continuation.
      cancelSquare(); ++squareRequest; squareController = new AbortController(); squareMoreLoading.value = false;
      squareItems.value = squareItems.value.filter((row) => row.id !== item.id);
      squareTotal.value = Math.max(0, squareTotal.value - 1);
      if (!result.queued && squareNextOffset.value !== null) squareNextOffset.value = Math.max(0, squareNextOffset.value - 1);
    }
  } catch (error) {
    operationNote.value = `收藏操作失败：${error.message || error}`;
    notifyOperation(operationNote.value, false);
  } finally { favoriteBusy.value = favoriteBusy.value.filter((id) => id !== item.id); }
}

async function loadPublishSources() {
  const [localPrompts, localCollections] = await Promise.all([
    listLocalPrompts({ query: "", categoryId: null }),
    listLocalCollections({ query: "", categoryId: null }),
  ]);
  return [
    ...localPrompts.map((item) => ({ ...item, kind: "prompt" })),
    ...localCollections.map((item) => ({ ...item, kind: "collection" })),
  ];
}

async function openPublish() {
  if (publishResume.value && publishSourcesLoading.value) return;
  const request = ++publishSourcesRequest;
  const token = getSession().accessToken;
  const current = () => request === publishSourcesRequest && publishResume.value && getSession().accessToken === token;
  operationNote.value = "";
  publishResume.value = true;
  publishSourcesLoading.value = true;
  publishSourcesError.value = '';
  publishSources.value = [];
  publishSourceId.value = '';
  try {
    const [sources] = await Promise.all([loadPublishSources(), loadRemoteCatalog()]);
    if (current()) publishSources.value = sources;
  } catch (error) {
    if (current()) publishSourcesError.value = `本地内容读取失败：${error.message || error}`;
  } finally { if (current()) publishSourcesLoading.value = false; }
}

async function startPublish() {
  if (!getSession().loggedIn) {
    pendingPublish.value = true;
    openLogin("发布需要登录");
    return;
  }
  await openPublish();
}

async function finishLogin() {
  session.value = getSession();
  sessionRestoreError.value = "";
  profileReady.value = false;
}

async function cancelProfile() {
  await logoutFromSettings();
  closeLoginPage();
}

async function finishProfile() {
  profileReady.value = true;
  if (!loginReason.value) return;
  loginReason.value = "";
  await refreshFavorites();
  if (space.value === 'square') await loadSquare();
  if (pendingPublish.value) {
    pendingPublish.value = false;
    await openPublish();
  }
}

async function submitPublish() {
  if (!publishSourceId.value || publishBusy.value || publishSourcesLoading.value || publishSourcesError.value || publishAssetsLoading.value || publishAssetsError.value) return;
  publishBusy.value = true;
  const source = publishSources.value.find((item) => item.id === publishSourceId.value);
  const account = getSession();
  const selectedAssets = publishAssets.value.filter(asset => publishAssetIds.value.includes(asset.id));
  const assertAccount = () => { if (!account.accessToken || getSession().accessToken !== account.accessToken) throw new Error('账号已变化，请重新确认公开附件'); };
  try {
    if (!source) throw new Error("未选择本地内容");
    assertAccount();
    let members;
    if (source.kind === 'collection') {
      const currentMembers = await listCollectionMembers(source.id);
      if (selectedAssets.some(asset => !currentMembers.some(member => member.id === asset.memberId))) throw new Error('所选附件的成员已移出合集，请重新选择本地内容');
      members = currentMembers.map(member => ({
        title: member.title, content: member.content, category_id: publicationCategory(member.category_id), model: member.model,
        ...(selectedAssets.some(asset => asset.memberId === member.id) ? { asset_ids: selectedAssets.filter(asset => asset.memberId === member.id).map(asset => asset.id) } : {}),
      }));
      if (!members.length) throw new Error('合集至少需要一条提示词才能发布');
      if (members.some(member => !member.title?.trim() || !member.content?.trim())) throw new Error('合集成员标题和正文不能为空');
    }
    if (selectedAssets.length > 12 || selectedAssets.reduce((total, asset) => total + assetSize(asset), 0) > 20 * 1024 * 1024) throw new Error('一份稿件最多选择 12 个附件、合计 20 MiB');
    const assetRefs = [];
    for (const asset of selectedAssets) {
      assertAccount();
      operationNote.value = `正在上传 ${assetRefs.length + 1}/${selectedAssets.length}：${asset.name}`;
      assetRefs.push(await uploadPrivateAsset(asset, account.accessToken));
    }
    assertAccount();
    const result = await publishWithQueue({
      sourceId: publishSourceId.value,
      title: source?.title,
      content: source?.content ?? "",
      categoryId: remoteCatalog.value ? publishCategoryId.value : publicationCategory(source?.category_id),
      model: remoteCatalog.value ? publishModel.value : source?.model,
      ...(source.kind === "collection" ? { kind: "collection", members } : {}),
      ...(assetRefs.length ? { assetRefs } : {}),
    });
    publishResume.value = false;
    operationNote.value = result.queued ? "草稿已保存在本机队列，尚未提交审核。" : "已提交审核，本地内容仍可编辑。";
    if (result.queueWarning) operationNote.value += ` 队列整理失败：${result.queueWarning}。请勿重复提交此稿。`;
    notifyOperation(operationNote.value, !result.queueWarning, 'publish');
  } catch (error) {
    operationNote.value = `发布失败：${error.message || error}`;
  } finally { publishBusy.value = false; }
}

async function refreshFavorites() {
  if (!getSession().loggedIn) {
    favoriteIds.value = [];
    return;
  }
  try {
    const account = getSession().email;
    const rows = squareItems.value;
    const ids = await applyQueuedFavorites(rows.filter(row => row.is_favorite).map(row => row.id));
    if (account === getSession().email && rows === squareItems.value) favoriteIds.value = ids;
  } catch {
    favoriteIds.value = [];
  }
}

let searchTimer;
let searchComposing = false;
function cancelSearch() { clearTimeout(searchTimer); }
function beginSearchComposition() { searchComposing = true; cancelSearch(); cancelSquare(); ++squareRequest; ++localRequest; }
async function finishSearchComposition() { searchComposing = false; await nextTick(); scheduleSearch(); }
function scheduleSearch(event) {
  cancelSearch();
  cancelSquare();
  ++squareRequest; ++localRequest;
  if (searchComposing || event?.isComposing) return;
  if (space.value === 'local') { reloadPrompts(); return; }
  squareLoading.value = true;
  if (!query.value.trim()) { loadSquare(); return; }
  searchTimer = setTimeout(() => loadSquare(), 250);
}
onUnmounted(() => { cancelSearch(); cancelSquare(); ++squareRequest; ++localRequest; ++catalogRequest; });

async function loadSquare(refreshCatalog = false) {
  cancelSearch();
  cancelSquare();
  const request = ++squareRequest;
  squareController = new AbortController();
  const signal = squareController.signal;
  squareItems.value = [];
  // Global category totals remain valid while a different filtered page loads.
  squareTotal.value = 0; squareNextOffset.value = null;
  squareMoreLoading.value = false; squareMoreError.value = false;
  failedReferenceImages.value = {};
  if (contentScroller.value) contentScroller.value.scrollTop = 0;
  squareOffline.value = false;
  squareBlocked.value = false;
  squareLoading.value = true;
  try {
  await refreshDownloaded();
  const access = await getLocalSetting("square_access");
  if (request !== squareRequest || space.value !== 'square') return;
  if (access === "0") {
    squareItems.value = [];
    squareBlocked.value = true;
    return;
  }
    if (refreshCatalog === true || !remoteCatalog.value) await loadRemoteCatalog();
    if (request !== squareRequest || space.value !== 'square') return;
    if (sortTab.value === '收藏' && !getSession().loggedIn) { openLogin('收藏需要登录'); return; }
    const page = await listSquarePage({ contentLanguage: contentLanguage.value, sort: sortTab.value, query: query.value, model: modelFilter.value, categoryId: selectedId.value, signal });
    if (request !== squareRequest || space.value !== 'square') return;
    squareItems.value = page.items;
    if (page.category_counts && Number.isInteger(page.category_total)) {
      squareCategoryCounts.value = page.category_counts; squareCategoryTotal.value = page.category_total;
    }
    squareTotal.value = page.total; squareNextOffset.value = page.next_offset;
    await refreshFavorites();
    rememberModels(squareItems.value);
  } catch {
    if (request !== squareRequest || space.value !== "square") return;
    squareItems.value = [];
    squareOffline.value = true;
  } finally {
    if (request === squareRequest) squareLoading.value = false;
  }
}

async function loadMoreSquare(retry = false) {
  if (space.value !== 'square' || hasTaskPage.value || squareLoading.value || squareMoreLoading.value || squareNextOffset.value === null || (squareMoreError.value && !retry)) return;
  const request = squareRequest, offset = squareNextOffset.value;
  squareMoreLoading.value = true; squareMoreError.value = false;
  try {
    const page = await listSquarePage({ contentLanguage: contentLanguage.value, sort: sortTab.value, query: query.value, model: modelFilter.value, categoryId: selectedId.value, offset, signal: squareController.signal });
    if (request !== squareRequest || space.value !== 'square') return;
    const ids = new Set(squareItems.value.map(item => item.id));
    squareItems.value = [...squareItems.value, ...page.items.filter(item => !ids.has(item.id) && ids.add(item.id))];
    squareTotal.value = page.total; squareNextOffset.value = page.next_offset;
    await refreshFavorites();
  } catch {
    if (request === squareRequest) squareMoreError.value = true;
  } finally { if (request === squareRequest) squareMoreLoading.value = false; }
}

let squareRequest = 0;

let catalogRequest = 0;
async function loadRemoteCatalog() {
  const request = ++catalogRequest;
  try {
    const catalog = await fetchSquareCatalog();
    if (request !== catalogRequest) return;
    const opened = new Set(remoteCategoryGroups.value.filter(item => item.open).map(item => item.id));
    const initial = remoteCatalog.value === null;
    remoteCatalog.value = catalog;
    remoteCategoryGroups.value = buildCategoryTree(catalog.categories.map(item => ({...item,is_system:true}))).map(group => ({...group,icon:catalog.categories.find(item=>item.id===group.id)?.icon || 'folder',open:initial ? group.open : opened.has(group.id)}));
    catalogError.value = '';
    if (space.value === 'square' && selectedId.value && !catalog.categories.some(item => item.id === selectedId.value)) selectedId.value = null;
    if (space.value === 'square' && modelFilter.value && !catalog.models.some(item=>item.id===modelFilter.value)) modelFilter.value = '';
  } catch (error) { if (request === catalogRequest) catalogError.value = error.message || '广场配置暂时不可用'; }
}
function remoteCategoryLabel(category) {
  const parent = remoteCatalog.value?.categories.find(item=>item.id===category.parent_id);
  return parent ? `${parent.name} / ${category.name}` : category.name;
}

function publicationCategory(id) {
  const category = categoryById(id);
  const parent = categoryById(category?.parent_id);
  if (remoteCatalog.value) {
    const enabled = new Set(remoteCatalog.value.categories.map(item=>item.id));
    return enabled.has(id) ? id : enabled.has(parent?.id) ? parent.id : null;
  }
  return category?.is_system ? category.id : parent?.is_system ? parent.id : null;
}

function rememberModels(items) {
  seenModels.value = parseModelNames(seenModels.value, items);
}

async function loadModelPrefs() {
  modelCatalogText.value = (await getLocalSetting("model_catalog")) || "";
  customModelsText.value = (await getLocalSetting("custom_models")) || "";
  defaultModel.value = (await getLocalSetting("default_model")) || "";
  showModelTags.value = (await getLocalSetting("show_model_tags")) !== "0";
  variableHints.value = (await getLocalSetting("variable_hints")) === "1";
  const storedLang = await getLocalSetting("ui_language");
  uiLanguage.value = storedLang === "en" ? "en" : "zh";
  document.documentElement.lang = uiLanguage.value === "en" ? "en" : "zh-CN";
  document.body.dataset.density = (await getLocalSetting('density')) === 'compact' ? 'compact' : 'comfortable';
}

async function applyUiLanguage(next) {
  uiLanguage.value = next === "en" ? "en" : "zh";
  document.documentElement.lang = uiLanguage.value === "en" ? "en" : "zh-CN";
  await setLocalSetting("ui_language", uiLanguage.value);
}

function onModelFilter() {
  if (space.value === "square") loadSquare();
}

function tabCount(tab) {
  if (space.value === "square") {
    return tab === sortTab.value ? squareTotal.value : 0;
  }
  const rows = modelFilter.value ? libraryItems.value.filter(item => item.kind === 'prompt' && item.model === modelFilter.value) : libraryItems.value;
  return filterLocalItems(rows, {
    tab,
    favoriteIds: localFavoriteIds.value,
  }).length;
}

let menuReturnFocus;
function closeContextMenu() {
  contextMenu.value = null;
  menuReturnFocus?.focus();
}
function onMenuKeydown(event) {
  const buttons = [...event.currentTarget.querySelectorAll('button:not(:disabled)')];
  const current = buttons.indexOf(document.activeElement);
  if (event.key === 'Tab') { closeContextMenu(); return; }
  if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key) || !buttons.length) return;
  event.preventDefault();
  const next = event.key === 'Home' ? 0 : event.key === 'End' ? buttons.length - 1 : (current + (event.key === 'ArrowDown' ? 1 : -1) + buttons.length) % buttons.length;
  buttons[next].focus();
}
function openContextMenu(event, item) {
  if (!contextActions(item).length) return;
  const rect = event.currentTarget?.getBoundingClientRect();
  menuReturnFocus = event.currentTarget?.matches?.('button') ? event.currentTarget : event.currentTarget?.querySelector?.('.prompt-title');
  contextMenu.value = { x: Math.max(8, Math.min(event.clientX || rect?.left || 8, window.innerWidth - 190)), y: Math.max(8, Math.min(event.clientY || rect?.bottom || 8, window.innerHeight - 300)), item };
  nextTick(() => document.querySelector('[data-testid="context-menu"] button')?.focus());
}

function contextActions(item) {
  if (space.value === "square") {
    return [
      { id: "details", label: '查看详情' },
      ...(downloadedIds.value.includes(item.id) && referenceImages(item).length ? [{ id: 'complete-images', label: '补全参考图', disabled: downloadBusy.value.includes(item.id) }] : []),
    ];
  }
  if (item.kind === "collection") {
    return [];
  }
  return [
    { id: "edit", label: t("edit") },
    { id: 'duplicate', label: '复制副本' },
    {
      id: "favorite",
      label: localFavoriteIds.value.includes(item.id) ? t("unfavorite") : t("favorite"),
    },
    { id: "delete", label: t("remove") },
  ];
}

async function runContextAction(action) {
  const item = contextMenu.value?.item;
  closeContextMenu();
  if (!item) return;
  if (action === 'details') { await openSquareDetail(item); return; }
  if (action === 'complete-images') { await completeImages(item); return; }
  if (action === "edit" || action === "open") {
    if (action === "edit") editing.value = item; else openItem(item);
    return;
  }
  if (action === "copy") { await quickCopy(item); return; }
  if (action === "duplicate") { await duplicatePrompt(item); return; }
  if (action === "use") {
    startUse(item);
    return;
  }
  if (action === "delete") {
    removePrompt(item.id);
    return;
  }
  if (action === "download") {
    await downloadSquare(item);
    return;
  }
  if (action === "favorite") {
    if (space.value === "square") {
      await favoriteSquare(item);
      return;
    }
    try {
      localFavoriteIds.value = await toggleLocalFavorite(item.id);
      notifyOperation(localFavoriteIds.value.includes(item.id) ? `「${item.title}」已收藏。` : `「${item.title}」已取消收藏。`, true);
    } catch (error) { notifyOperation(`收藏失败：${error.message || error}`, false); }
  }
}

function openPublicationsFromSettings() {
  pendingNavigation.value = () => { publicationsOpen.value = true; };
  settingsView.value?.requestClose();
}

async function closeSettings() {
  settingsPage.value = 'general';
  logoutError.value = '';
  settingsOpen.value = false;
  await refreshLocalSettings();
  finishNavigation();
}

async function refreshLocalSettings() {
  await loadLauncherShortcut();
  categoryGroups.value = buildCategoryTree(await listLocalCategories());
  if (selectedId.value && selectedId.value !== "__uncategorized__" && !categoryById(selectedId.value)) selectedId.value = null;
  const storedTheme = (await getLocalSetting("theme")) || "light";
  if (["light", "dark", "system"].includes(storedTheme) && storedTheme !== theme.value) await applyTheme(storedTheme);
  await loadModelPrefs();
  await reloadPrompts();
}

function setSort(tab) {
  sortTab.value = tab;
  if (space.value === "square") loadSquare();
}

function openSquare() {
  if (contentKind.value === "skills") { contentKind.value = "prompts"; if (space.value === "square") return; }
  operationNote.value = '';
  cancelSearch(); ++localRequest;
  space.value = "square";
  if (selectedId.value === "__uncategorized__" || (selectedId.value && !categoryById(selectedId.value)?.is_system)) {
    selectedId.value = null;
  }
  sortTab.value = "推荐";
  loadSquare(true);
}

function openLocal() {
  if (contentKind.value === "skills") { contentKind.value = "prompts"; if (space.value === "local") return; }
  operationNote.value = '';
  cancelSearch(); cancelSquare(); ++squareRequest;
  space.value = "local";
  if (selectedId.value && selectedId.value !== '__uncategorized__' && !categoryById(selectedId.value)) selectedId.value = null;
  sortTab.value = "全部";
  squareOffline.value = false;
  reloadPrompts();
}

function toggleTheme() {
  applyTheme(dark.value ? "light" : "dark");
}

let systemTheme;
function syncSystemTheme() {
  if (theme.value !== 'system') return;
  dark.value = Boolean(systemTheme?.matches);
  document.body.classList.toggle('theme-dark', dark.value);
}
onMounted(() => {
  systemTheme = window.matchMedia?.('(prefers-color-scheme: dark)');
  systemTheme?.addEventListener?.('change', syncSystemTheme);
});
onUnmounted(() => systemTheme?.removeEventListener?.('change', syncSystemTheme));

async function applyTheme(next, persist = true) {
  if (persist) await setLocalSetting("theme", next);
  theme.value = next;
  const prefersDark =
    typeof window !== "undefined" && Boolean(window.matchMedia?.("(prefers-color-scheme: dark)")?.matches);
  dark.value = next === "dark" || (next === "system" && prefersDark);
  document.body.classList.toggle("theme-dark", dark.value);
}

async function reloadPrompts() {
  if (space.value !== "local") return;
  const request = ++localRequest;
  const filter = { query: query.value, categoryId: selectedId.value };
  const [filteredPrompts, filteredCollections, allPrompts, allCollections] = await Promise.all([
    listLocalPrompts(filter),
    listLocalCollections(filter),
    listLocalPrompts({ query: "", categoryId: null }),
    listLocalCollections({ query: "", categoryId: null }),
  ]);
  if (request !== localRequest || space.value !== "local") return;
  prompts.value = filteredPrompts;
  collections.value = filteredCollections;
  allLocalItems.value = [...allPrompts, ...allCollections];
  downloadedIds.value = [...new Set(allPrompts.map(row => row.remote_id).filter(Boolean))];
  emit("library-changed", allPrompts.length);
}

let localRequest = 0;

function closeEditor() {
  pendingDelete.value = null;
  creating.value = false;
  editing.value = null;
  editorError.value = "";
  finishNavigation();
}

function coverPreview(item) {
  return parseCoverUrls(item.cover_json).slice(0, 3);
}

function cardCategory(item) {
  if (!item.category_id) return '';
  return (space.value === 'square' ? remoteCatalog.value?.categories.find(c => c.id === item.category_id) : categoryById(item.category_id))?.name ?? '';
}

async function savePrompt({ id, kind, title, content, categoryId, model, coverType, coverUrls, assets }) {
  if (editorBusy.value) return;
  editorBusy.value = true;
  editorError.value = "";
  let saved = false;
  try {
    if (id && kind === "collection") {
      await updateLocalCollection({ id, title, categoryId, coverType, coverUrls });
    } else if (id) {
      await updateLocalPrompt({ id, title, content, categoryId, model, assets });
    } else if (kind === "collection") {
      await createLocalCollection({ title, categoryId, coverType, coverUrls });
    } else {
      await createLocalPrompt({ title, content, categoryId, model, assets });
    }
    saved = true;
    closeEditor();
    notifyOperation(`已保存「${title}」。`, true, 'save');
    await refreshOperationView();
    if (reading.value?.id === id) reading.value = (await listLocalPrompts()).find(row => row.id === id) || null;
  } catch (error) {
    if (saved) notifyOperation(`已保存，但刷新失败：${error.message || error}`, false, 'save', true);
    else editorError.value = `保存失败：${error.message || error}`;
  } finally { editorBusy.value = false; }
}

function removePrompt(id) {
  if (editorBusy.value) return;
  const item = editing.value?.id === id ? editing.value : reading.value?.id === id ? reading.value : prompts.value.find(row => row.id === id);
  if (!item) return;
  operationNotice.value = null;
  pendingDelete.value = { id, title: item.title, kind: item.kind };
  nextTick(() => cancelDeleteButton.value?.focus());
}

async function confirmRemovePrompt() {
  if (!pendingDelete.value || editorBusy.value) return;
  const { id, title, kind } = pendingDelete.value;
  editorBusy.value = true;
  let deleted = false;
  try {
    if (kind === "collection") await deleteLocalCollection(id);
    else await deleteLocalPrompt(id);
    deleted = true;
    pendingDelete.value = null;
    notifyOperation(`已删除「${title}」。${kind === 'collection' ? '合集内的提示词已保留。' : ''}`, true, 'delete');
    if (editing.value?.id === id) closeEditor();
    if (reading.value?.id === id) reading.value = null;
    if (openedCollection.value?.id === id) openedCollection.value = null;
    else if (openedCollection.value) await openCollection(openedCollection.value);
    await reloadPrompts();
  } catch (error) {
    editorError.value = `${deleted ? '已删除，但刷新失败' : '删除失败'}：${error.message || error}`;
    notifyOperation(editorError.value, false, 'delete');
  }
  finally { editorBusy.value = false; }
}

async function finishUse(text) {
  if (useBusy.value) return;
  const prompt = using.value;
  useBusy.value = true;
  useError.value = "";
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    useError.value = "复制失败，请检查剪贴板权限后重试；填写内容已保留。";
    useBusy.value = false;
    return;
  }
  if (using.value?.id === prompt.id) using.value = null;
  try {
    await recordLocalPromptUse(prompt.id);
  } catch (error) {
    notifyOperation(`已复制，但保存使用记录失败：${error.message || error}。无需再次复制。`, false, 'copy');
    useBusy.value = false;
    return;
  }
  notifyOperation(`已复制「${prompt.title}」。`, true, 'copy');
  try { await refreshOperationView(); }
  catch (error) { notifyOperation(`已复制，但刷新失败：${error.message || error}`, false, 'copy', true); }
  finally { useBusy.value = false; }
}

async function openDownloaded(item) {
  try {
    const rows = await listLocalPrompts();
    const row = rows.find(p => p.remote_id === item.id);
    if (!row) { await refreshDownloaded(); notifyOperation('本地副本已移除，可以重新下载。', false); return; }
    const collection = item.kind === 'collection' && row.collection_id
      ? (await listLocalCollections()).find(c => c.id === row.collection_id) : null;
    closeSquareDetail();
    openLocal();
    if (collection) await openCollection(collection); else reading.value = row;
  } catch (error) { notifyOperation(`打开失败：${error.message || error}`, false); }
}

async function quickCopy(item) {
  if (useBusy.value) return;
  useBusy.value = true;
  let copied = false;
  try {
    await navigator.clipboard.writeText(item.content); copied = true;
    await recordLocalPromptUse(item.id);
    notifyOperation(`已复制「${item.title}」。`, true, 'copy');
    await refreshOperationView();
  } catch (error) { notifyOperation(`${copied ? '已复制，但记录或刷新失败' : '复制失败'}：${error.message || error}`, false, 'copy'); }
  finally { useBusy.value = false; }
}

async function duplicatePrompt(item) {
  if (editorBusy.value) return;
  editorBusy.value = true;
  let saved = false;
  try {
    const assets = item.asset_count ? await listPromptAssets(item.id) : [];
    await createLocalPrompt({ title: `${item.title}（副本）`, content: item.content, categoryId: item.category_id, model: item.model, assets: assets.map(a => ({ ...a, id: crypto.randomUUID() })) });
    saved = true; await reloadPrompts();
    notifyOperation('已创建副本。', true, 'save');
  } catch (error) { notifyOperation(`${saved ? '已创建副本，但刷新失败' : '复制副本失败'}：${error.message || error}`, false, 'save', saved); }
  finally { editorBusy.value = false; }
}

function openItem(item) {
  if (space.value === "square") {
    openSquareDetail(item);
    return;
  }
  if (item.kind === "collection") {
    openCollection(item);
    return;
  }
  reading.value = item;
}

async function openCollection(collection) {
  const request = ++collectionRequest;
  const isCurrent = () => request === collectionRequest && openedCollection.value?.id === collection.id;
  openedCollection.value = collection;
  collectionError.value = "";
  collectionLoading.value = true;
  collectionReady.value = false;
  collectionMembers.value = [];
  collectionCandidates.value = [];
  try {
    const [members, candidates] = await Promise.all([
      listCollectionMembers(collection.id), listLocalPrompts({ query: "", categoryId: null }),
    ]);
    if (!isCurrent()) return false;
    collectionMembers.value = members;
    collectionCandidates.value = candidates;
    collectionReady.value = true;
    return true;
  } catch (error) {
    if (isCurrent()) collectionError.value = `读取失败：${error.message || error}`;
    return false;
  } finally { if (isCurrent()) collectionLoading.value = false; }
}

async function addToOpenedCollection(promptIds) {
  if (collectionBusy.value || !collectionReady.value || !openedCollection.value) return;
  collectionBusy.value = true; collectionError.value = '';
  const collection = openedCollection.value, failed = [];
  let count = 0;
  for (const id of promptIds) {
    try { await addPromptToCollection(id, collection.id); count++; }
    catch (error) { failed.push(`${collectionCandidates.value.find(p => p.id === id)?.title || id}：${error.message || error}`); }
  }
  if (count) {
    const refreshed = await openCollection(collection);
    try { await reloadPrompts(); }
    catch (error) { notifyOperation(`已加入合集，但刷新失败：${error.message || error}`, false, 'collection', true); }
    if (!refreshed) notifyOperation(`已加入合集，但刷新失败：${collectionError.value}`, false, 'collection', true);
    else notifyOperation(`已加入合集，共 ${count} 条${failed.length ? `；${failed.length} 条失败，请重试剩余项` : ''}。`, !failed.length, 'collection');
  }
  if (failed.length) collectionError.value = `加入失败：${failed.join('；')}`;
  collectionBusy.value = false;
}

async function removeFromOpenedCollection(promptId) {
  await changeCollectionMember(promptId, false);
}

async function changeCollectionMember(promptId, adding) {
  if (collectionBusy.value || !collectionReady.value || !openedCollection.value) return;
  collectionBusy.value = true;
  collectionError.value = "";
  let written = false;
  const completed = adding ? '已加入合集' : '已移出合集';
  try {
    await (adding ? addPromptToCollection : removePromptFromCollection)(promptId, openedCollection.value.id);
    written = true;
    notifyOperation(`${completed}。`, true, 'collection');
    if (!await openCollection(openedCollection.value)) throw new Error(collectionError.value);
    await reloadPrompts();
  } catch (error) {
    if (written) notifyOperation(`${completed}，但刷新失败：${error.message || error}`, false, 'collection', true);
    else collectionError.value = `${adding ? '加入' : '移除'}失败：${error.message || error}`;
  }
  finally { collectionBusy.value = false; }
}

function openCollectionMember(member) {
  reading.value = member;
}

function useCollectionMember(member) {
  startUse(member);
}

function startUse(member) {
  useError.value = "";
  using.value = member;
}

function editOpenedCollection() {
  editing.value = { ...openedCollection.value, kind: "collection" };
}

onMounted(async () => {
  applyHostChrome(document.body, props.host);
  await loadLauncherShortcut();
  const stored = await getLocalSetting("theme");
  if (stored === "dark" || stored === "light" || stored === "system") {
    await applyTheme(stored);
  }
  categoryGroups.value = buildCategoryTree(await listLocalCategories());
  await loadModelPrefs();
  localFavoriteIds.value = await listLocalFavoriteIds();
  await reloadPrompts();
  await refreshFavorites();
  if (window.__TAURI_INTERNALS__) {
    const { listen } = await import("@tauri-apps/api/event");
    extraLauncherUnlisteners.push(await listen('local-library-changed', () => { reloadPrompts().catch(() => {}); }));
    extraLauncherUnlisteners.push(await listen('launcher-navigate', ({payload}) => {
      const action = () => {
        if (payload?.destination === 'ai-settings') {settingsPage.value='models';settingsOpen.value=true;}
        else if (payload?.destination === 'square-detail' && payload.id) {openSquareDetail({id:payload.id,title:'广场提示词'});}
      };
      if(settingsOpen.value) {pendingNavigation.value=action;settingsView.value?.requestClose();} else navigateTo(action);
    }));
    await listen("open-new-prompt", () => {
      if (!settingsOpen.value) navigateTo(() => { creating.value = true; });
    });
  }
});
</script>

<style scoped>
.empty-actions { display: flex; flex-wrap: wrap; justify-content: center; gap: 10px; }
.prompt-card.is-selected { outline: 1px solid var(--text); outline-offset: -1px; }
.prompt-card.as-row:has(.card-selection) { position: relative; padding-left: 48px; }
.prompt-card.as-row .card-selection { position: absolute; left: 16px; top: 50%; transform: translateY(-50%); margin: 0; font-size: 0; }
.card-selection { display: flex; align-items: center; gap: 6px; color: var(--muted); font-size: 12px; margin-bottom: 10px; }
.card-selection input { width: 16px; height: 16px; }
.card-more { margin-left: auto; font-size: 18px; }
.title-tool { white-space: nowrap; }
.browse-pagination { display: flex; justify-content: center; align-items: center; gap: 20px; padding: 24px 0; color: var(--muted); font-size: 12px; }
.publication-files { border: 1px solid var(--line); border-radius: 12px; padding: 16px; margin: 20px 0; }
.publication-image-preview { width: 80px; height: 60px; object-fit: cover; border-radius: 6px; flex-shrink: 0; }
.publication-image-summary { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; }
.publication-image-summary p { width: 100%; margin: 0; color: var(--muted); font-size: 12px; }
.publication-files legend { font-size: 13px; font-weight: 600; padding: 0 6px; }
.publication-file { display: flex; gap: 12px; align-items: center; padding: 12px 0; }
.publication-file input { width: 16px; height: 16px; flex: none; }
.publication-file span { min-width: 0; overflow-wrap: anywhere; }
.publication-file small { display: block; margin-top: 4px; font-size: 12px; color: var(--muted); }
</style>
