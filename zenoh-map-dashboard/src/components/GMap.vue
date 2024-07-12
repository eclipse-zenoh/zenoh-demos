<template>
  <!-- {{ markers }} -->
  <GoogleMap
    ref="mapRef"
    api-key="TOKEN GOES HERE"
    style="width: 100%; height: 45vw"
    :zoom="16"
    :center="center"
    :renderingType="'VECTOR'"
    :streetViewControl="false"
    :rotateControl="false"
    :scaleControl="false"
    :mapTypeControl="false"
    :zoomControl="false"
  >
    <CustomMarker
      v-for="marker in markers"
      :key="marker.id"
      :options="{ position: marker.position, anchorPoint: 'BOTTOM_CENTER' }"
    >
      <div class="d-flex flex-column justify-content-center align-items-center">
        <div class="d-sm-inline-flex border rounded-pill bg-light text-dark">
          <div class="badge badge-secondary">
            <div style="font-size: small">Driver: {{ marker.id }} Speed: {{ marker.speed }}</div>
          </div>
        </div>
        <FontAwesomeIcon v-if="(marker.kind==='car')" icon="car" size="xl" :style="{ 'color': marker.color }"/>
        <FontAwesomeIcon v-else-if="(marker.kind==='motorbike')" icon="motorcycle" size="xl" :style="{ 'color': marker.color }" />
        <FontAwesomeIcon v-else icon="fa-solid fa-circle-question" size="xl" :style="{ 'color': marker.color }" />
      </div>
    </CustomMarker>
  </GoogleMap>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { GoogleMap, CustomMarker } from 'vue3-google-map'
import { Vehicle } from '../types/Vehicle'
import { Region } from '../utils/utils'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

type Props = {
  markers: Array<Vehicle>
}

const props = defineProps<Props>()
const center = ref({ lat: 48.864716, lng: 2.349014 })
const mapRef = ref(null)

watch(
  () => props.markers,
  (new_markers) => {
    const points = new_markers.map((x) => x.position)
    const reg = new Region(points)
    if (points.length > 0)
      center.value = reg.centroid()
  }
)

// onMounted(() => {})
// the map should expose the center and the markers as parameters
</script>
