<script setup lang="ts">
import { askConfirmation } from '@/lib/confirm';
import { computed, onMounted, onUnmounted, shallowRef, ref } from 'vue';
import MarkdownIt from 'markdown-it';
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import { openUrl } from '@tauri-apps/plugin-opener';
import { relaunch } from '@tauri-apps/plugin-process';
import { Update } from '@tauri-apps/plugin-updater';
import { Download, RefreshCw, ShieldCheck, ExternalLink, ScrollText, Copy } from '@lucide/vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { useWorkspaceStore } from '@/stores/workspace';
const store = useWorkspaceStore();
const version = ref('读取中…');
const versionReady = ref(false);
async function copyVersion() {
  if (!versionReady.value) return;
  try {
    await navigator.clipboard.writeText(version.value);
    store.notify('版本号已复制');
  } catch (error) {
    store.notify(`复制版本号失败：${String(error)}`);
  }
}
const checking = ref(false);
const installing = ref(false);
const progress = ref(0);
const updateAvailable = shallowRef<Update | null>(null);
const status = ref('');
const releaseNotes = ref('');
const releaseNotesLoading = ref(false);
const releaseNotesError = ref(false);
const markdown = new MarkdownIt({ html: false, linkify: true, breaks: true });
const renderedNotes = computed(() => markdown.render(releaseNotes.value));
async function openReleaseLink(event: MouseEvent) {
  const link = (event.target as HTMLElement).closest('a');
  if (!link) return;
  event.preventDefault();
  try {
    const url = new URL(link.getAttribute('href') || '', 'https://github.com/yeyongzhi/Lumo-Hub/');
    if (url.protocol === 'https:' || url.protocol === 'http:') await openUrl(url.href);
  } catch (error) {
    store.notify(`无法打开链接：${String(error)}`);
  }
}
async function openRelease() {
  if (!updateAvailable.value) return;
  try {
    await openUrl(
      `https://github.com/yeyongzhi/Lumo-Hub/releases/tag/v${encodeURIComponent(updateAvailable.value.version)}`,
    );
  } catch (error) {
    store.notify(`无法打开发布页面：${String(error)}`);
  }
}
async function openChangelog() {
  try {
    await openUrl('https://yeyongzhi.github.io/Lumo-Hub/changelog');
  } catch (error) {
    status.value = `无法打开更新日志：${String(error)}`;
  }
}
async function openProject() {
  try {
    await openUrl('https://github.com/yeyongzhi/Lumo-Hub');
  } catch (error) {
    status.value = `无法打开项目主页：${String(error)}`;
  }
}
onMounted(async () => {
  try {
    version.value = await getVersion();
    versionReady.value = true;
  } catch {
    version.value = '未知版本';
  }
});
async function checkUpdate() {
  checking.value = true;
  status.value = '';
  try {
    await updateAvailable.value?.close();
    updateAvailable.value = null;
    releaseNotes.value = '';
    releaseNotesError.value = false;
    const metadata = await invoke<ConstructorParameters<typeof Update>[0] | null>(
      'check_app_update',
    );
    updateAvailable.value = metadata ? new Update(metadata) : null;
    status.value = updateAvailable.value
      ? `发现新版本 ${updateAvailable.value.version}`
      : '当前已是最新版本。';
    if (updateAvailable.value) {
      releaseNotes.value =
        updateAvailable.value.body || String(updateAvailable.value.rawJson?.notes || '');
      releaseNotesLoading.value = true;
      try {
        releaseNotes.value = await invoke<string>('get_release_notes', {
          version: updateAvailable.value.version,
        });
      } catch {
        releaseNotesError.value = true;
      } finally {
        releaseNotesLoading.value = false;
      }
    }
  } catch (error) {
    status.value = /404|not found/i.test(String(error))
      ? '当前发布源尚未提供更新清单，请稍后重试。'
      : `检查更新失败：${String(error)}，可再次点击检查更新重试。`;
  } finally {
    checking.value = false;
  }
}
onUnmounted(() => {
  void updateAvailable.value?.close().catch(() => {});
});
async function installUpdate() {
  if (
    !updateAvailable.value ||
    !(await askConfirmation(
      `将 Lumo-Hub 更新到 ${updateAvailable.value.version}。用户数据会保留。现在安装吗？`,
    ))
  )
    return;
  installing.value = true;
  let received = 0;
  let total = 0;
  try {
    await updateAvailable.value.downloadAndInstall((event) => {
      if (event.event === 'Started') {
        total = event.data.contentLength ?? 0;
      } else if (event.event === 'Progress') {
        received += event.data.chunkLength;
        if (total > 0) progress.value = Math.min(100, Math.round((received / total) * 100));
      } else if (event.event === 'Finished') {
        progress.value = 100;
      }
    });
    await relaunch();
  } catch (error) {
    status.value = `安装更新失败：${String(error)}`;
  } finally {
    installing.value = false;
  }
}
defineExpose({ checkUpdate });
</script>

<template>
  <div class="relative flex flex-col items-start gap-5 text-left">
    <img src="/brand/lumohub.png" alt="LumoHub" class="size-20 shrink-0 object-contain" />
    <div>
      <div class="flex flex-wrap items-center gap-x-3 gap-y-2">
        <h2 class="text-2xl font-semibold">Lumo-Hub</h2>
        <div class="flex items-center gap-1">
          <Badge class="px-3 py-1 text-sm">版本 {{ version }}</Badge>
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label="复制版本号"
            :disabled="!versionReady"
            @click="copyVersion"
            ><Copy
          /></Button>
        </div>
      </div>
      <p class="mt-1 text-sm text-muted-foreground">本地优先的个人桌面效率工作台</p>
    </div>
    <div class="flex flex-wrap items-center gap-3">
      <Button :disabled="checking || installing" @click="checkUpdate"
        ><RefreshCw data-icon="inline-start" :class="checking ? 'animate-spin' : ''" />{{
          checking ? '正在检查…' : '检查更新'
        }}</Button
      ><Button
        v-if="updateAvailable"
        variant="outline"
        :disabled="installing"
        @click="installUpdate"
        ><Download data-icon="inline-start" />{{
          installing ? `正在下载 ${progress}%` : '下载并安装'
        }}</Button
      ><Button variant="outline" @click="openChangelog"
        ><ScrollText data-icon="inline-start" />查看更新日志</Button
      ><Button variant="outline" @click="openProject"
        ><ExternalLink data-icon="inline-start" />项目主页</Button
      >
    </div>
    <p v-if="status" class="max-w-xl text-sm text-muted-foreground">{{ status }}</p>
    <div
      v-if="updateAvailable"
      class="w-full max-w-3xl rounded-2xl border border-border bg-muted/50 p-5 text-left"
    >
      <div class="flex items-center gap-2 font-medium">
        <ShieldCheck class="size-4 text-primary" />版本 {{ updateAvailable.version }}
      </div>
      <p class="mt-1 text-xs text-muted-foreground">GitHub Release 更新内容</p>
      <p v-if="releaseNotesLoading" class="mt-4 text-sm text-muted-foreground">正在读取更新内容…</p>
      <template v-else>
        <p v-if="releaseNotesError" class="mt-4 text-sm text-muted-foreground">
          无法读取 GitHub Release，以下显示更新清单中的说明。
        </p>
        <div
          v-if="releaseNotes"
          class="release-notes mt-4 break-words text-sm leading-6"
          @click="openReleaseLink"
          v-html="renderedNotes"
        />
        <p v-else class="mt-4 text-sm text-muted-foreground">此版本没有附带更新说明。</p>
        <Button variant="link" class="mt-3 h-auto p-0" @click="openRelease"
          ><ExternalLink data-icon="inline-start" />查看 GitHub Release</Button
        >
      </template>
    </div>
  </div>
</template>

<style scoped>
.release-notes :deep(h1),
.release-notes :deep(h2),
.release-notes :deep(h3) {
  margin: 1rem 0 0.5rem;
  font-weight: 600;
}
.release-notes :deep(p),
.release-notes :deep(ul),
.release-notes :deep(ol) {
  margin: 0.5rem 0;
}
.release-notes :deep(ul) {
  list-style: disc;
  padding-left: 1.5rem;
}
.release-notes :deep(ol) {
  list-style: decimal;
  padding-left: 1.5rem;
}
.release-notes :deep(a) {
  color: var(--primary);
  text-decoration: underline;
}
.release-notes :deep(pre) {
  overflow-x: auto;
  white-space: pre-wrap;
}
</style>
