/**
 * Short Link Form Component
 */

import { Component, createEffect, createSignal, For, Show } from 'solid-js';
import { ShortLink } from '../../types';
import {
    canChangeSlug,
    validateParamKey,
    validateParamValue,
    validateRedirect,
    validateSlug
} from '../../utils/validation.utils';
import './ShortLinkForm.css';

interface ShortLinkFormProps {
  link?: ShortLink;
  author: string;
  onSubmit: (data: {
    slug: string;
    params: Record<string, string>;
    author: string;
    redirect: string | null;
  }) => void;
  onCancel: () => void;
  isSubmitting: boolean;
}

const ShortLinkForm: Component<ShortLinkFormProps> = (props) => {
  const isEditing = () => !!props.link;
  const canEditSlug = () => !props.link || canChangeSlug(props.link.created_at);

  const [slug, setSlug] = createSignal(props.link?.slug || '');
  const [redirect, setRedirect] = createSignal(props.link?.redirect || '');
  const [params, setParams] = createSignal<Array<{ key: string; value: string }>>(
    props.link
      ? Object.entries(props.link.params).map(([key, value]) => ({ key, value }))
      : [{ key: '', value: '' }]
  );

  const [errors, setErrors] = createSignal<{
    slug?: string;
    redirect?: string;
    params?: Record<number, { key?: string; value?: string }>;
  }>({});

  // Auto-add new parameter row when user starts typing in the last row
  createEffect(() => {
    const currentParams = params();
    if (currentParams.length > 0) {
      const lastParam = currentParams[currentParams.length - 1];
      // If the last row has any content, add a new empty row
      if (lastParam.key || lastParam.value) {
        addParam();
      }
    }
  });

  const addParam = () => {
    setParams([...params(), { key: '', value: '' }]);
  };

  const removeParam = (index: number) => {
    setParams(params().filter((_, i) => i !== index));
  };

  const updateParam = (index: number, field: 'key' | 'value', value: string) => {
    const updated = [...params()];
    updated[index][field] = value;
    setParams(updated);
  };

  const validate = (): boolean => {
    const newErrors: typeof errors extends () => infer R ? R : never = {};

    // Validate slug
    const slugValidation = validateSlug(slug());
    if (!slugValidation.valid) {
      newErrors.slug = slugValidation.error;
    }

    // Validate redirect
    const redirectValidation = validateRedirect(redirect());
    if (!redirectValidation.valid) {
      newErrors.redirect = redirectValidation.error;
    }

    // Validate params
    const paramErrors: Record<number, { key?: string; value?: string }> = {};
    params().forEach((param, index) => {
      if (param.key || param.value) {
        const keyValidation = validateParamKey(param.key);
        if (!keyValidation.valid) {
          paramErrors[index] = { ...paramErrors[index], key: keyValidation.error };
        }

        const valueValidation = validateParamValue(param.value);
        if (!valueValidation.valid) {
          paramErrors[index] = { ...paramErrors[index], value: valueValidation.error };
        }
      }
    });

    if (Object.keys(paramErrors).length > 0) {
      newErrors.params = paramErrors;
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: Event) => {
    e.preventDefault();

    if (!validate()) {
      return;
    }

    // Convert params array to object, filter out empty entries
    const paramsObject: Record<string, string> = {};
    params().forEach(param => {
      if (param.key && param.value) {
        paramsObject[param.key] = param.value;
      }
    });

    props.onSubmit({
      slug: slug().trim(),
      params: paramsObject,
      author: props.author,
      redirect: redirect().trim() || null,
    });
  };

  return (
    <form class="short-link-form" onSubmit={handleSubmit}>
      {/* Slug */}
      <div class="form-group">
        <label class="form-label">
          Slug *
          <Show when={!canEditSlug()}>
            <span class="form-hint">(Cannot be changed after creation)</span>
          </Show>
        </label>
        <input
          type="text"
          class="form-input"
          classList={{ 'form-input-error': !!errors().slug }}
          value={slug()}
          onInput={(e) => setSlug(e.currentTarget.value)}
          placeholder="my-link"
          disabled={!canEditSlug()}
          required
        />
        <Show when={errors().slug}>
          <span class="form-error">{errors().slug}</span>
        </Show>
        <span class="form-hint">
          Allowed: a-z, 0-9, -, _, ., ~ (2-50 characters)
        </span>
      </div>

      {/* Redirect URL */}
      <div class="form-group">
        <label class="form-label">
          Redirect URL (Optional)
        </label>
        <input
          type="url"
          class="form-input"
          classList={{ 'form-input-error': !!errors().redirect }}
          value={redirect()}
          onInput={(e) => setRedirect(e.currentTarget.value)}
          placeholder="https://example.com"
        />
        <Show when={errors().redirect}>
          <span class="form-error">{errors().redirect}</span>
        </Show>
        <span class="form-hint">
          Must be a valid HTTPS URL. Leave empty to use default landing page.
        </span>
      </div>

      {/* Author (read-only) */}
      <div class="form-group">
        <label class="form-label">Author</label>
        <input
          type="text"
          class="form-input"
          value={props.author}
          disabled
        />
      </div>

      {/* Parameters */}
      <div class="form-group">
        <div class="params-header">
          <label class="form-label">Parameters</label>
          <button
            type="button"
            class="btn btn-secondary btn-sm"
            onClick={addParam}
          >
            + Add Parameter
          </button>
        </div>
        <div class="params-list">
          <For each={params()}>
            {(param, index) => (
              <div class="param-row">
                <div class="param-inputs">
                  <div class="param-input-wrapper">
                    <input
                      type="text"
                      class="form-input"
                      classList={{
                        'form-input-error': !!errors().params?.[index()]?.key
                      }}
                      value={param.key}
                      onInput={(e) => updateParam(index(), 'key', e.currentTarget.value)}
                      placeholder="key"
                    />
                    <Show when={errors().params?.[index()]?.key}>
                      <span class="form-error">{errors().params![index()].key}</span>
                    </Show>
                  </div>
                  <div class="param-input-wrapper">
                    <input
                      type="text"
                      class="form-input"
                      classList={{
                        'form-input-error': !!errors().params?.[index()]?.value
                      }}
                      value={param.value}
                      onInput={(e) => updateParam(index(), 'value', e.currentTarget.value)}
                      placeholder="value"
                    />
                    <Show when={errors().params?.[index()]?.value}>
                      <span class="form-error">{errors().params![index()].value}</span>
                    </Show>
                  </div>
                </div>
                <button
                  type="button"
                  class="btn btn-danger btn-icon"
                  onClick={() => removeParam(index())}
                  disabled={params().length === 1}
                  title="Remove parameter"
                >
                  ×
                </button>
              </div>
            )}
          </For>
        </div>
        <span class="form-hint">
          Keys must start with letter/underscore, contain only alphanumeric/underscore (max 50 chars).
          Values max 200 chars.
        </span>
      </div>

      {/* Actions */}
      <div class="form-actions">
        <button
          type="button"
          class="btn btn-secondary"
          onClick={props.onCancel}
          disabled={props.isSubmitting}
        >
          Cancel
        </button>
        <button
          type="submit"
          class="btn btn-primary"
          disabled={props.isSubmitting}
        >
          {props.isSubmitting
            ? 'Saving...'
            : isEditing()
              ? 'Update Link'
              : 'Create Link'
          }
        </button>
      </div>
    </form>
  );
};

export default ShortLinkForm;
