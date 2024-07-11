<template>
  <!-- {{ markers }} -->
  <GoogleMap
    ref="mapRef"
    api-key="API KEY GOES HERE"
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
      :options="{ position: marker.position, anchorPoint: 'CENTER' }"
    >
      <div>
        <div>{{ marker.speed }}</div>
        <div
          style="height: 10px; width: 10px; border-radius: 50%"
          :style="{ 'background-color': marker.color }"
        ></div>
      </div>
    </CustomMarker>
  </GoogleMap>
</template>

<script setup lang="ts">
import { defineProps, ref, watch } from 'vue'
import { GoogleMap, CustomMarker } from 'vue3-google-map'
import { CarData } from '../types/CarData'
import { Region } from '../utils/utils'

type Props = {
  markers: Array<CarData>
}

const props = defineProps<Props>()
const center = ref({ lat: 48.864716, lng: 2.349014 })
const mapRef = ref(null)

watch(
  () => props.markers,
  (new_markers) => {
    const points = new_markers.map((x) => x.position)
    const reg = new Region(points)
    center.value = reg.centroid()
  }
)

// onMounted(() => {})
// the map should expose the center and the markers as parameters
</script>
