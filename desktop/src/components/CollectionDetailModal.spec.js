import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import CollectionDetailModal from "./CollectionDetailModal.vue";

describe("CollectionDetailModal", () => {
  it('retains page focus after confirmed membership disables the submit button', async () => {
    const w = mount(CollectionDetailModal, { attachTo: document.body, props: {
      collection: { id: 'c', title: 'Collection' }, prompts: [{ id: 'p', title: 'Prompt' }],
    } });
    await w.get('select').setValue('p');
    const add = w.findAll('button').find(button => button.text() === '加入合集');
    add.element.focus();
    await add.trigger('click');
    expect(w.get('select').element.value).toBe('p');
    await w.setProps({ prompts: [{ id: 'p', title: 'Prompt', collection_id: 'c' }] });
    await flushPromises();
    expect(document.activeElement).toBe(w.get('[role="region"]').element);
    await w.get('[role="region"]').trigger('keydown', { key: 'Escape' });
    expect(w.emitted('cancel')).toHaveLength(1);
    w.unmount();
  });
  it('explains an empty collection and keeps member actions separate from the title', async () => {
    const w = mount(CollectionDetailModal, { props: { collection: { id: 'c', title: '工作' } } });
    expect(w.get('.collection-empty').text()).toContain('从下方选择');
    expect(w.get('select').text()).toContain('暂无可加入');
    const member = { id: 'p', title: '长标题'.repeat(20), collection_id: 'c' };
    await w.setProps({ members: [member], prompts: [member] });
    expect(w.find('.collection-empty').exists()).toBe(false);
    await w.get('.member-title').trigger('click');
    expect(w.emitted('open')[0][0]).toEqual(member);
    await w.get('[data-testid="remove-member"]').trigger('click');
    expect(w.emitted('remove-member')[0][0]).toBe('p');
    w.unmount();
  });
  it("opens a sparse grid with real images and placeholders", () => {
    const w = mount(CollectionDetailModal, {
      props: {
        collection: {
          id: "col-1",
          title: "人像灵感",
          cover_type: "grid",
          cover_json: JSON.stringify(["one.jpg", "two.jpg", "three.jpg"]),
        },
        members: [],
        prompts: [],
      },
    });
    const cells = w.findAll('[data-testid="cover-grid"] i');
    expect(cells).toHaveLength(9);
    expect(w.findAll('[data-testid="cover-grid"] img')).toHaveLength(3);
    expect(cells.filter((cell) => cell.classes().includes("filled"))).toHaveLength(3);
    expect(w.get('[data-testid="collection-detail"]').exists()).toBe(true);
  });
});
