import { flushPromises } from '@vue/test-utils';
import SearchableSelect from '../components/SearchableSelect.vue';
export function selectComponent(wrapper, id) { return wrapper.findAllComponents(SearchableSelect).find(item => item.vm.$attrs['data-testid'] === id); }
export async function selectOption(wrapper, id, value) {
  const component = selectComponent(wrapper,id);
  const option = component.props('options').find(item => item.value === value);
  if(!option) throw new Error(`Missing ${id} option ${value}`);
  await component.get('button').trigger('click');
  const input = document.querySelector('.select-popup input');
  input.value = option.label; input.dispatchEvent(new Event('input',{bubbles:true}));
  await flushPromises();
  const choice = [...document.querySelectorAll('.select-popup [role=option]')].find(item => item.textContent === option.label);
  choice.click(); await flushPromises();
}
