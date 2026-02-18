# FEAT-010: Property Tax and Revenue Overhaul

**Category:** Feature / Economy
**Priority:** T2
**Source:** community_wishlists.md -- Section 3.1, master_architecture.md

## Summary

Replace per-citizen flat tax with property tax on assessed building value. Millage rate system with separate rates for residential, commercial, industrial. Progressive taxation option. Bond/debt financing for capital projects. Per-product/resource taxes.

## Details

- Property tax: assessed_value * millage_rate per zone type
- Assessment ratio (fraction of market value taxed)
- Sales tax, income tax as additional revenue sources
- Bond issuance for capital projects with interest payments
- Tax policy visibly affects growth and business investment
- Tax increment financing (TIF) districts

## Dependencies

- Land value system (provides assessed values)
- Building system (building values)

## Acceptance Criteria

- [ ] Property tax replaces per-citizen flat tax
- [ ] Separate millage rates per zone type
- [ ] Bond issuance functional
- [ ] Tax changes visibly affect city growth
