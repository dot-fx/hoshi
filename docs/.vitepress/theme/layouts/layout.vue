<template>
  <div :class="{ 'is-landing': frontmatter.customLayout === 'landing' }">
    <div v-if="frontmatter.customLayout === 'landing'">
      <landing />
    </div>
    <DefaultTheme.Layout v-else />
  </div>
</template>

<script setup>
import { useData } from 'vitepress'
import DefaultTheme from 'vitepress/theme'
import MyLanding from '../components/landing.vue'

const { frontmatter } = useData()
</script>

<style>
.is-landing .VPContent {
  padding: 0 !important;
  width: 100% !important;
  max-width: 100% !important;
}

.is-landing .VPDoc {
  padding: 0 !important;
  margin: 0 !important;
}


body, html {
  margin: 0 !important;
  padding: 0 !important;
}
</style>