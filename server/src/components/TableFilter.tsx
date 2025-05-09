import { useState } from "react";
import { IconSearch, IconFilter } from "@tabler/icons-react";
import { TableFilterProps } from "../../types";

export const TableFilter = ({
  searchTerm,
  setSearchTerm,
  searchPlaceholder = "Search...",
  filters,
  setFilters,
  filterCategories = [],
  activeFilterCategory = "",
  setActiveFilterCategory,
}: TableFilterProps) => {
  const [isFilterMenuOpen, setIsFilterMenuOpen] = useState(false);

  const isMultiCategory = filterCategories.length > 0;

  const handleSingleCategoryFilterChange = (key: string) => {
    if (!isMultiCategory) {
      setFilters((prev: Record<string, boolean>) => ({
        ...prev,
        [key]: !prev[key],
      }));
    }
  };

  const handleMultiCategoryFilterChange = (category: string, key: string) => {
    if (isMultiCategory && setActiveFilterCategory) {
      setFilters((prev: Record<string, Record<string, boolean>>) => ({
        ...prev,
        [category]: {
          ...prev[category],
          [key]: !prev[category][key],
        },
      }));
    }
  };

  return (
    <div className="flex items-center justify-between gap-2 mb-4 pt-1 px-1 z-50">
      <div className="relative flex-1">
        <div className="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
          <IconSearch className="text-white" size={18} />
        </div>
        <input
          type="text"
          className="bg-secondarybg text-white w-full pl-10 pr-4 py-2 rounded-lg focus:outline-none focus:ring-1 focus:ring-gray-600"
          placeholder={searchPlaceholder}
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
      </div>

      <div className="relative">
        <button
          onClick={() => setIsFilterMenuOpen(!isFilterMenuOpen)}
          className="p-2 rounded-lg bg-white text-black hover:bg-white/80 cursor-pointer transition-colors flex items-center gap-1 border-accentx"
        >
          <IconFilter size={20} />
          <span>Filter</span>
        </button>

        {isFilterMenuOpen && (
          <div className="absolute right-0 mt-2 w-64 rounded-md shadow-lg bg-secondarybg ring-1 ring-black ring-opacity-5 z-[100]">
            <div className="py-1 px-2">
              {isMultiCategory && (
                <div className="flex border-b border-accentx mb-2">
                  {filterCategories.map((category) => (
                    <button
                      key={category}
                      className={`px-3 py-2 text-sm font-medium ${
                        activeFilterCategory === category
                          ? "text-white border-b-2 border-white"
                          : "text-gray-400 hover:text-white"
                      }`}
                      onClick={() =>
                        setActiveFilterCategory &&
                        setActiveFilterCategory(category)
                      }
                    >
                      {category.charAt(0).toUpperCase() + category.slice(1)}
                    </button>
                  ))}
                </div>
              )}

              {isMultiCategory && setActiveFilterCategory ? (
                <>
                  <div className="p-2 text-sm font-medium text-white">
                    {activeFilterCategory.charAt(0).toUpperCase() +
                      activeFilterCategory.slice(1)}
                  </div>

                  <div className="max-h-60 overflow-y-auto">
                    {Object.keys(filters[activeFilterCategory] || {}).map(
                      (filterValue) => (
                        <div
                          key={filterValue}
                          className="flex items-center px-3 py-2"
                        >
                          <input
                            type="checkbox"
                            id={`${activeFilterCategory}-${filterValue}`}
                            checked={
                              (
                                filters[activeFilterCategory] as Record<
                                  string,
                                  boolean
                                >
                              )[filterValue] !== false
                            }
                            onChange={() =>
                              handleMultiCategoryFilterChange(
                                activeFilterCategory,
                                filterValue
                              )
                            }
                            className="form-checkbox h-4 w-4 mr-2"
                          />
                          <label
                            htmlFor={`${activeFilterCategory}-${filterValue}`}
                            className="ml-2 text-sm text-white cursor-pointer flex-1 truncate"
                            title={filterValue}
                          >
                            {filterValue}
                          </label>
                        </div>
                      )
                    )}
                  </div>
                </>
              ) : (
                <>
                  <div className="p-2 text-sm font-medium text-white">
                    Event Types
                  </div>
                  <div className="max-h-60 overflow-y-auto">
                    {Object.keys(filters).map((filterKey) => (
                      <div
                        key={filterKey}
                        className="flex items-center px-3 py-2"
                      >
                        <input
                          type="checkbox"
                          id={filterKey}
                          checked={
                            (filters as Record<string, boolean>)[filterKey] !==
                            false
                          }
                          onChange={() =>
                            handleSingleCategoryFilterChange(filterKey)
                          }
                          className="form-checkbox h-4 w-4 mr-2"
                        />
                        <label
                          htmlFor={filterKey}
                          className="ml-2 text-sm text-white cursor-pointer flex-1"
                        >
                          {filterKey}
                        </label>
                      </div>
                    ))}
                  </div>
                </>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
