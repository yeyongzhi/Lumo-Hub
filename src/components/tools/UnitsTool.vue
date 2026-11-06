<script setup lang="ts">
import { computed, ref } from 'vue';
import { ArrowLeftRight } from '@lucide/vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Field, FieldLabel } from '@/components/ui/field';
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { convertUnit, unitCategories, unitsFor, type UnitCategory } from '@/lib/unit-conversion';

const category = ref<UnitCategory>('length');
const amount = ref<number | string>(1);
const from = ref('m');
const to = ref('cm');
const units = computed(() => unitsFor(category.value));
const result = computed(() => {
  if (String(amount.value).trim() === '') return '';
  const value = convertUnit(Number(amount.value), category.value, from.value, to.value);
  if (!Number.isFinite(value)) return '';
  return String(Number(value.toPrecision(12)));
});
function changeCategory(value: unknown) {
  if (
    typeof value !== 'string' ||
    !Object.prototype.hasOwnProperty.call(unitCategories, value) ||
    value === category.value
  )
    return;
  category.value = value as UnitCategory;
  [from.value, to.value] = Object.keys(units.value);
}
function swap() {
  [from.value, to.value] = [to.value, from.value];
}
</script>

<template>
  <div class="flex flex-col gap-5">
    <Tabs :model-value="category" @update:model-value="changeCategory">
      <TabsList>
        <TabsTrigger v-for="(group, id) in unitCategories" :key="id" :value="id">{{
          group.label
        }}</TabsTrigger>
      </TabsList>
    </Tabs>
    <div class="grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-end gap-3">
      <Field>
        <FieldLabel for="unit-from">原单位</FieldLabel>
        <Select v-model="from">
          <SelectTrigger id="unit-from" class="w-full"><SelectValue /></SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectItem v-for="(unit, id) in units" :key="id" :value="id">{{
                unit[0]
              }}</SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
      </Field>
      <Button variant="outline" size="icon" title="交换单位" aria-label="交换单位" @click="swap"
        ><ArrowLeftRight
      /></Button>
      <Field>
        <FieldLabel for="unit-to">目标单位</FieldLabel>
        <Select v-model="to">
          <SelectTrigger id="unit-to" class="w-full"><SelectValue /></SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectItem v-for="(unit, id) in units" :key="id" :value="id">{{
                unit[0]
              }}</SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
      </Field>
    </div>
    <div class="grid grid-cols-2 gap-3">
      <Field>
        <FieldLabel for="unit-amount">数值</FieldLabel>
        <Input id="unit-amount" v-model="amount" type="number" step="any" placeholder="输入数值" />
      </Field>
      <Field>
        <FieldLabel for="unit-result">换算结果</FieldLabel>
        <Input id="unit-result" :model-value="result" readonly />
      </Field>
    </div>
    <p v-if="category === 'data'" class="text-sm text-muted-foreground">
      KB、MB 等按 1000 进制换算；KiB、MiB 等按 1024 进制换算。
    </p>
  </div>
</template>
